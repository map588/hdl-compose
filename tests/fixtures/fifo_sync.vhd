library ieee;
use ieee.std_logic_1164.all;

entity fifo_sync is
  generic (
    DEPTH : integer := 256;
    WIDTH : integer := 8
  );
  port (
    clk   : in  std_logic;
    rst_n : in  std_logic;
    din   : in  std_logic_vector(WIDTH-1 downto 0);
    wr_en : in  std_logic;
    rd_en : in  std_logic;
    dout  : out std_logic_vector(WIDTH-1 downto 0);
    full  : out std_logic;
    empty : out std_logic
  );
end entity fifo_sync;

architecture rtl of fifo_sync is
begin
  -- stub
end architecture rtl;
