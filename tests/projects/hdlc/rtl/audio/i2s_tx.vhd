
library ieee;
use ieee.std_logic_1164.all;

entity i2s is
  generic (
    mclk_sclk_ratio  : integer := 4; --number of mclk periods per sclk period
    sclk_lrclk_ratio : integer := 64; --number of sclk periods per word select period
    d_width          : integer := 24); --data width
  port (
    resetn  : in std_logic; --asynchronous active low reset
    mclk    : in std_logic; --master clock
    sclk    : out std_logic; --serial clock (or bit clock)
    lrclk   : out std_logic; --word select (or left-right clock)
    sdata   : out std_logic; --serial data transmit
    data_tx : in std_logic_vector(d_width - 1 downto 0)); --channel data to transmit
end entity;

architecture logic of i2s is

  signal sclk_int    : std_logic := '0'; --internal serial clock signal
  signal lrclk_int   : std_logic := '0'; --internal word select signal
  signal data_tx_int : std_logic_vector(d_width - 1 downto 0); --internal left channel tx data buffer

begin

  process (mclk, resetn)
    variable sclk_cnt  : integer := 0; --counter of master clocks during half period of serial clock
    variable lrclk_cnt : integer := 0; --counter of serial clock toggles during half period of word select
  begin

    if (resetn = '0') then --asynchronous reset
      sclk_cnt  := 0; --clear mclk/sclk counter
      lrclk_cnt := 0; --clear sclk/lrclk counter
      sclk_int    <= '0'; --clear serial clock signal
      lrclk_int   <= '0'; --clear word select signal
      data_tx_int <= (others => '0'); --clear internal channel tx data buffer
      sdata       <= '0'; --clear serial data transmit output
    elsif rising_edge(mclk) then --master clock rising edge
      if (sclk_cnt < mclk_sclk_ratio / 2 - 1) then --less than half period of sclk
        sclk_cnt := sclk_cnt + 1; --increment mclk/sclk counter
      else --half period of sclk
        sclk_cnt := 0; --reset mclk/sclk counter
        sclk_int <= not sclk_int; --toggle serial clock
        if (lrclk_cnt < sclk_lrclk_ratio - 1) then --less than half period of lrclk
          lrclk_cnt := lrclk_cnt + 1; --increment sclk/lrclk counter
          if (sclk_int = '1' and lrclk_cnt < d_width * 2 + 3) then --falling edge of sclk during data word
            sdata       <= data_tx_int(d_width - 1); --transmit serial data bit 
            data_tx_int <= data_tx_int(d_width - 2 downto 0) & '0'; --shift data of right channel tx data buffer
          end if;
        else --half period of lrclk
          lrclk_cnt := 0; --reset sclk/lrclk counter
          lrclk_int   <= not lrclk_int; --toggle word select
          data_tx_int <= data_tx; --latch in right channel data to transmit
        end if;
      end if;
    end if;
  end process;

  sclk  <= sclk_int; --output serial clock
  lrclk <= lrclk_int; --output word select

end architecture;
