
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

entity enveloper is
  generic (
    -- CC/param addresses are generics because the VCA set (73/75/70/72) is
    -- not contiguous; the VCF instance uses 80/81/82/83.
    ATTACK_ADDR   : integer range 0 to 127 := 73;
    DECAY_ADDR    : integer range 0 to 127 := 75;
    SUSTAIN_ADDR  : integer range 0 to 127 := 70;
    RELEASE_ADDR  : integer range 0 to 127 := 72;
    CLKS_PER_TICK : integer               := 104167 -- ~1 ms envelope tick
  );
  port (
    clk         : in  std_logic;
    rst_n       : in  std_logic;
    gate        : in  std_logic; -- note held (note_states bit)
    trig        : in  std_logic; -- retrigger pulse (triggers bit)
    param_write : in  std_logic;
    param_addr  : in  std_logic_vector(6 downto 0);
    param_data  : in  std_logic_vector(6 downto 0);
    gate_out    : out std_logic;
    env_out     : out std_logic_vector(11 downto 0)
  );
end entity;

architecture Behavioral of enveloper is

  component map_reg is
    generic (
      n            : integer  := 7;
      addr_size    : integer  := 7;
      address      : integer;
      output_width : positive := 12
    );
    port (
      D    : in  STD_LOGIC_VECTOR(n - 1 downto 0);
      ADDR : in  STD_LOGIC_VECTOR(addr_size - 1 downto 0);
      CLK  : in  STD_LOGIC;
      LOAD : in  STD_LOGIC;
      RSTN : in  STD_LOGIC;
      Q    : out STD_LOGIC_VECTOR(output_width - 1 downto 0)
    );
  end component;

  type state_type is (IDLE, ATTACK, DECAY, SUSTAIN, REL);

  signal state : state_type := IDLE;

  -- Higher CC value = slower A/D/R, so store the inverted value; the
  -- range_mapper inside map_reg then scales 7 -> 12 bits (x32).
  signal param_inv : std_logic_vector(6 downto 0);

  signal step_a      : std_logic_vector(11 downto 0);
  signal step_d      : std_logic_vector(11 downto 0);
  signal sustain_lvl : std_logic_vector(11 downto 0);
  signal step_r      : std_logic_vector(11 downto 0);

  signal envelope  : unsigned(11 downto 0) := (others => '0');
  signal tick      : std_logic := '0';
  signal clk_count : integer range 0 to CLKS_PER_TICK - 1 := 0;

  -- midi_processor's triggers output is a LEVEL (held until the next note
  -- message), so retrigger on its rising edge only
  signal trig_prev : std_logic := '0';

  -- A zero step would stall the FSM forever
  function at_least_one(s : std_logic_vector) return unsigned is
  begin
    if unsigned(s) = 0 then
      return to_unsigned(1, s'length);
    else
      return unsigned(s);
    end if;
  end function;

begin

  param_inv <= std_logic_vector(127 - unsigned(param_data));

  attack_reg: map_reg
    generic map (n => 7, addr_size => 7, address => ATTACK_ADDR, output_width => 12)
    port map (D => param_inv, ADDR => param_addr, CLK => clk, LOAD => param_write, RSTN => rst_n, Q => step_a);

  decay_reg: map_reg
    generic map (n => 7, addr_size => 7, address => DECAY_ADDR, output_width => 12)
    port map (D => param_inv, ADDR => param_addr, CLK => clk, LOAD => param_write, RSTN => rst_n, Q => step_d);

  sustain_reg: map_reg
    generic map (n => 7, addr_size => 7, address => SUSTAIN_ADDR, output_width => 12)
    port map (D => param_data, ADDR => param_addr, CLK => clk, LOAD => param_write, RSTN => rst_n, Q => sustain_lvl);

  release_reg: map_reg
    generic map (n => 7, addr_size => 7, address => RELEASE_ADDR, output_width => 12)
    port map (D => param_inv, ADDR => param_addr, CLK => clk, LOAD => param_write, RSTN => rst_n, Q => step_r);

  tick_gen: process (clk)
  begin
    if rising_edge(clk) then
      if rst_n = '0' then
        clk_count <= 0;
        tick      <= '0';
      elsif clk_count = CLKS_PER_TICK - 1 then
        clk_count <= 0;
        tick      <= '1';
      else
        clk_count <= clk_count + 1;
        tick      <= '0';
      end if;
    end if;
  end process;

  fsm: process (clk)
    variable step : unsigned(11 downto 0);
    variable sum  : unsigned(12 downto 0);
  begin
    if rising_edge(clk) then
      if rst_n = '0' then
        state     <= IDLE;
        envelope  <= (others => '0');
        trig_prev <= '0';
      else
        trig_prev <= trig;
        -- Retrigger (same-note note-on or voice steal) restarts the attack
        -- from the current level, any state, without waiting for a tick.
        if trig = '1' and trig_prev = '0' and gate = '1' then
          state <= ATTACK;
        elsif tick = '1' then
          case state is
            when IDLE =>
              if gate = '1' then
                state <= ATTACK;
              end if;

            when ATTACK =>
              if gate = '0' then
                state <= REL;
              else
                step := at_least_one(step_a);
                sum  := ('0' & envelope) + step;
                if sum >= to_unsigned(4095, 13) then
                  envelope <= (others => '1');
                  state    <= DECAY;
                else
                  envelope <= sum(11 downto 0);
                end if;
              end if;

            when DECAY =>
              if gate = '0' then
                state <= REL;
              else
                step := at_least_one(step_d);
                if envelope <= unsigned(sustain_lvl) + step then
                  envelope <= unsigned(sustain_lvl);
                  state    <= SUSTAIN;
                else
                  envelope <= envelope - step;
                end if;
              end if;

            when SUSTAIN =>
              if gate = '0' then
                state <= REL;
              else
                -- track live sustain-knob changes
                envelope <= unsigned(sustain_lvl);
              end if;

            when REL =>
              if gate = '1' then
                state <= ATTACK;
              else
                step := at_least_one(step_r);
                if envelope <= step then
                  envelope <= (others => '0');
                  state    <= IDLE;
                else
                  envelope <= envelope - step;
                end if;
              end if;
          end case;
        end if;
      end if;
    end if;
  end process;

  gate_out <= '0' when state = IDLE else '1';
  env_out  <= std_logic_vector(envelope);

end architecture;
