library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

entity mapped_reg is
  generic (
    n            : integer  := 7;
    addr_size    : integer  := 7;
    address      : integer;
    output_width : positive := 2
  );
  port (
    D    : in  STD_LOGIC_VECTOR(n - 1 downto 0);
    ADDR : in  STD_LOGIC_VECTOR(addr_size - 1 downto 0);
    CLK  : in  STD_LOGIC;
    LOAD : in  STD_LOGIC;
    RSTN : in  STD_LOGIC;
    Q    : out STD_LOGIC_VECTOR(output_width - 1 downto 0)
  );
end entity;

architecture rtl of mapped_reg is
  constant assigned_address : unsigned(addr_size - 1 downto 0) := to_unsigned(address, addr_size);

  component addressed_reg is
    generic (
      n         : integer := 7;
      addr_size : integer := 7;
      address   : integer
    );
    port (
      D    : in  STD_LOGIC_VECTOR(n - 1 downto 0);
      ADDR : in  STD_LOGIC_VECTOR(addr_size - 1 downto 0);
      CLK  : in  STD_LOGIC;
      LOAD : in  STD_LOGIC;
      RST  : in  STD_LOGIC;
      Q    : out STD_LOGIC_VECTOR(n - 1 downto 0)
    );
  end component;

  component range_mapper is
    generic (
      INPUT_WIDTH  : positive := n;
      OUTPUT_WIDTH : positive := output_width
    );
    port (
      din  : in  STD_LOGIC_VECTOR(INPUT_WIDTH - 1 downto 0);
      dout : out STD_LOGIC_VECTOR(OUTPUT_WIDTH - 1 downto 0)
    );
  end component;

  -- Synchronizer registers
  signal load_meta : std_logic := '0';
  signal load_sync : std_logic := '0';

  -- State tracking
  signal addr_matched   : std_logic := '0';
  signal load_valid     : std_logic := '0';
  signal cond_load      : std_logic := '0';
  signal input_original : std_logic_vector(n - 1 downto 0);
  signal rst            : std_logic;

  -- Prevent optimization of synchronizer chain
  attribute ASYNC_REG              : string;
  attribute ASYNC_REG of load_meta : signal is "TRUE";
  attribute ASYNC_REG of load_sync : signal is "TRUE";

begin
  rst <= not RSTN;

  -- Two-stage synchronizer with state maintenance

  sync_proc: process (CLK)
  begin
    if rising_edge(CLK) then
      if rst = '1' then
        load_meta <= '0';
        load_sync <= '0';
      else
        -- Two-stage synchronizer
        load_meta <= LOAD;
        load_sync <= load_meta;
      end if;
    end if;
  end process;

  -- Address matching and load control

  addr_proc: process (CLK)
  begin
    if rising_edge(CLK) then
      if rst = '1' then
        addr_matched <= '0';
        cond_load <= '0';
        load_valid <= '0';
      else
        -- Address match detection
        if unsigned(ADDR) = assigned_address then
          addr_matched <= '1';
        else
          addr_matched <= '0';
        end if;

        -- Generate load enable when address matches and load is valid
        load_valid <= addr_matched and load_sync;

        cond_load <= load_valid;
      end if;
    end if;
  end process;

  -- Component instantiations
  addr_reg: addressed_reg
    generic map (
      n         => n,
      addr_size => addr_size,
      address   => address
    )
    port map (
      D    => D,
      CLK  => CLK,
      ADDR => ADDR,
      LOAD => cond_load,
      RST  => rst,
      Q    => input_original
    );

  mapping: range_mapper
    generic map (
      INPUT_WIDTH  => n,
      OUTPUT_WIDTH => output_width
    )
    port map (
      din  => input_original,
      dout => Q
    );

end architecture;
