
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

-- Asynchronous AXI-Stream FIFO with gray-coded pointers (Cummings style).
-- Pure-VHDL replacement for the dual-clock axis_data_fifo IP. Depth 4 is
-- ample: crossing 96 kHz audio samples between the 100 MHz system clock and
-- the I2S mclk domain.

entity axis_cdc is
  generic (
    WIDTH : positive := 24
  );
  port (
    src_clk  : in  std_logic;
    src_rstn : in  std_logic;
    s_tdata  : in  std_logic_vector(WIDTH - 1 downto 0);
    s_tvalid : in  std_logic;
    s_tready : out std_logic;

    dst_clk  : in  std_logic;
    dst_rstn : in  std_logic;
    m_tdata  : out std_logic_vector(WIDTH - 1 downto 0);
    m_tvalid : out std_logic;
    m_tready : in  std_logic
  );
end entity;

architecture Behavioral of axis_cdc is

  constant DEPTH     : integer := 4;
  constant PTR_WIDTH : integer := 3; -- address bits + wrap bit

  type mem_type is array (0 to DEPTH - 1) of std_logic_vector(WIDTH - 1 downto 0);
  signal mem : mem_type := (others => (others => '0'));

  function bin2gray(b : unsigned) return unsigned is
  begin
    return b xor shift_right(b, 1);
  end function;

  -- write domain
  signal wptr_bin  : unsigned(PTR_WIDTH - 1 downto 0) := (others => '0');
  signal wptr_gray : unsigned(PTR_WIDTH - 1 downto 0) := (others => '0');
  signal rgray_m, rgray_s : unsigned(PTR_WIDTH - 1 downto 0) := (others => '0');

  -- read domain
  signal rptr_bin  : unsigned(PTR_WIDTH - 1 downto 0) := (others => '0');
  signal rptr_gray : unsigned(PTR_WIDTH - 1 downto 0) := (others => '0');
  signal wgray_m, wgray_s : unsigned(PTR_WIDTH - 1 downto 0) := (others => '0');

  signal full_int  : std_logic;
  signal empty_int : std_logic;

  attribute ASYNC_REG : string;
  attribute ASYNC_REG of rgray_m, rgray_s, wgray_m, wgray_s : signal is "TRUE";

begin

  -- full: write gray equals read gray (synced) with the two MSBs inverted
  full_int <= '1' when wptr_gray = (not rgray_s(PTR_WIDTH - 1 downto PTR_WIDTH - 2) & rgray_s(PTR_WIDTH - 3 downto 0)) else '0';
  empty_int <= '1' when rptr_gray = wgray_s else '0';

  s_tready <= not full_int;
  m_tvalid <= not empty_int;
  m_tdata  <= mem(to_integer(rptr_bin(1 downto 0)));

  wr_proc: process (src_clk)
    variable wnext : unsigned(PTR_WIDTH - 1 downto 0);
  begin
    if rising_edge(src_clk) then
      if src_rstn = '0' then
        wptr_bin  <= (others => '0');
        wptr_gray <= (others => '0');
        rgray_m   <= (others => '0');
        rgray_s   <= (others => '0');
      else
        rgray_m <= rptr_gray;
        rgray_s <= rgray_m;
        if s_tvalid = '1' and full_int = '0' then
          mem(to_integer(wptr_bin(1 downto 0))) <= s_tdata;
          wnext     := wptr_bin + 1;
          wptr_bin  <= wnext;
          wptr_gray <= bin2gray(wnext);
        end if;
      end if;
    end if;
  end process;

  rd_proc: process (dst_clk)
    variable rnext : unsigned(PTR_WIDTH - 1 downto 0);
  begin
    if rising_edge(dst_clk) then
      if dst_rstn = '0' then
        rptr_bin  <= (others => '0');
        rptr_gray <= (others => '0');
        wgray_m   <= (others => '0');
        wgray_s   <= (others => '0');
      else
        wgray_m <= wptr_gray;
        wgray_s <= wgray_m;
        if m_tready = '1' and empty_int = '0' then
          rnext     := rptr_bin + 1;
          rptr_bin  <= rnext;
          rptr_gray <= bin2gray(rnext);
        end if;
      end if;
    end if;
  end process;

end architecture;
