
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

-- I2C master for the MCP4728 quad 12-bit DAC.
-- Continuously repeats the Fast Write command (C2 C1 = 00), which loads all
-- four channel input registers in one 9-byte transfer:
--   [addr+W] [00 PD1 PD0 D11..D8]_A [D7..D0]_A ... same pair for B, C, D.
-- LDAC is held low so DAC outputs follow the input registers.
-- SDA is open-drain (drive 0 / release); the slave ACK is not checked.
-- SCL is driven push-pull: single master and MCP4728 does not clock-stretch.
-- Board wiring: DAC_SCLK pin = SCL, DAC_MOSI pin = SDA, DAC_SS pin = LDAC.
-- SDA (and ideally SCL) need pull-up resistors - present on common MCP4728
-- breakout boards; otherwise enable FPGA pull-ups in the XDC.

entity i2c_cv_dac is
  generic (
    -- clk cycles per I2C quarter-phase: SCL = clk / (CLK_DIV * 4)
    -- 104 MHz / (260 * 4) = 100 kHz -> full 4-channel refresh ~1 kHz
    CLK_DIV  : integer := 260;
    DEV_ADDR : std_logic_vector(6 downto 0) := "1100000"
  );
  port (
    clk   : in    std_logic;
    rst_n : in    std_logic;
    ch_a  : in    std_logic_vector(11 downto 0); -- VCA CV voice 0
    ch_b  : in    std_logic_vector(11 downto 0); -- VCA CV voice 1
    ch_c  : in    std_logic_vector(11 downto 0); -- VCF CV
    scl   : out   std_logic;
    sda   : inout std_logic;
    ldac  : out   std_logic
  );
end entity;

architecture Behavioral of i2c_cv_dac is

  type byte_array is array (0 to 8) of std_logic_vector(7 downto 0);
  type state_type is (S_IDLE, S_START, S_BIT, S_ACK, S_STOP);

  signal frame    : byte_array := (others => (others => '0'));
  signal state    : state_type := S_IDLE;
  signal qcnt     : integer range 0 to CLK_DIV - 1 := 0;
  signal qphase   : integer range 0 to 3 := 0;
  signal byte_idx : integer range 0 to 8 := 0;
  signal bit_idx  : integer range 0 to 7 := 7;
  signal sda_low  : std_logic := '0'; -- 1 = pull SDA low
  signal scl_int  : std_logic := '1';

begin

  ldac <= '0';
  sda  <= '0' when sda_low = '1' else 'Z';
  scl  <= scl_int;

  process (clk)
  begin
    if rising_edge(clk) then
      if rst_n = '0' then
        state   <= S_IDLE;
        qcnt    <= 0;
        qphase  <= 0;
        sda_low <= '0';
        scl_int <= '1';
      elsif qcnt /= CLK_DIV - 1 then
        qcnt <= qcnt + 1;
      else
        qcnt <= 0;
        case state is

          when S_IDLE =>
            -- latch inputs; unused channel D written as 0
            frame(0) <= DEV_ADDR & '0';
            frame(1) <= "0000" & ch_a(11 downto 8);
            frame(2) <= ch_a(7 downto 0);
            frame(3) <= "0000" & ch_b(11 downto 8);
            frame(4) <= ch_b(7 downto 0);
            frame(5) <= "0000" & ch_c(11 downto 8);
            frame(6) <= ch_c(7 downto 0);
            frame(7) <= (others => '0');
            frame(8) <= (others => '0');
            state  <= S_START;
            qphase <= 0;

          when S_START =>
            case qphase is
              when 0 => sda_low <= '0'; scl_int <= '1'; qphase <= 1;
              when 1 => sda_low <= '1'; qphase <= 2; -- SDA falls while SCL high
              when 2 => scl_int <= '0'; qphase <= 3;
              when others =>
                byte_idx <= 0;
                bit_idx  <= 7;
                state    <= S_BIT;
                qphase   <= 0;
            end case;

          when S_BIT =>
            case qphase is
              when 0 => -- set data while SCL low
                if frame(byte_idx)(bit_idx) = '1' then
                  sda_low <= '0';
                else
                  sda_low <= '1';
                end if;
                qphase <= 1;
              when 1 => scl_int <= '1'; qphase <= 2;
              when 2 => qphase <= 3;
              when others =>
                scl_int <= '0';
                qphase  <= 0;
                if bit_idx = 0 then
                  state <= S_ACK;
                else
                  bit_idx <= bit_idx - 1;
                end if;
            end case;

          when S_ACK =>
            case qphase is
              when 0 => sda_low <= '0'; qphase <= 1; -- release SDA for slave ACK
              when 1 => scl_int <= '1'; qphase <= 2;
              when 2 => qphase <= 3;                 -- ACK bit on SDA here (ignored)
              when others =>
                scl_int <= '0';
                qphase  <= 0;
                if byte_idx = 8 then
                  state <= S_STOP;
                else
                  byte_idx <= byte_idx + 1;
                  bit_idx  <= 7;
                  state    <= S_BIT;
                end if;
            end case;

          when S_STOP =>
            case qphase is
              when 0 => sda_low <= '1'; qphase <= 1; -- SDA low while SCL low
              when 1 => scl_int <= '1'; qphase <= 2;
              when 2 => sda_low <= '0'; qphase <= 3; -- SDA rises while SCL high
              when others =>
                state  <= S_IDLE;
                qphase <= 0;
            end case;

        end case;
      end if;
    end if;
  end process;

end architecture;
