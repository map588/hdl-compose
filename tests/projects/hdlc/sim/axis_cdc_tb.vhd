
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

-- 100 MHz writer, ~12.5 MHz reader: all samples must arrive in order.

entity axis_cdc_tb is
end entity;

architecture sim of axis_cdc_tb is

  constant N : integer := 50;

  signal src_clk  : std_logic := '0';
  signal dst_clk  : std_logic := '0';
  signal rstn     : std_logic := '0';
  signal s_tdata  : std_logic_vector(23 downto 0) := (others => '0');
  signal s_tvalid : std_logic := '0';
  signal s_tready : std_logic;
  signal m_tdata  : std_logic_vector(23 downto 0);
  signal m_tvalid : std_logic;
  signal m_tready : std_logic := '1';

  signal done : boolean := false;

begin

  src_clk <= not src_clk after 5 ns when not done else '0';
  dst_clk <= not dst_clk after 40 ns when not done else '0';

  dut: entity work.axis_cdc
    generic map (WIDTH => 24)
    port map (
      src_clk => src_clk, src_rstn => rstn,
      s_tdata => s_tdata, s_tvalid => s_tvalid, s_tready => s_tready,
      dst_clk => dst_clk, dst_rstn => rstn,
      m_tdata => m_tdata, m_tvalid => m_tvalid, m_tready => m_tready);

  writer: process
  begin
    wait for 100 ns;
    rstn <= '1';
    wait for 100 ns;
    for i in 1 to N loop
      s_tdata  <= std_logic_vector(to_unsigned(i * 37, 24));
      s_tvalid <= '1';
      wait until rising_edge(src_clk) and s_tready = '1';
    end loop;
    s_tvalid <= '0';
    wait;
  end process;

  reader: process
    variable expected : integer := 1;
  begin
    wait until rstn = '1';
    while expected <= N loop
      wait until rising_edge(dst_clk);
      if m_tvalid = '1' and m_tready = '1' then
        assert unsigned(m_tdata) = to_unsigned(expected * 37, 24)
          report "sample " & integer'image(expected) & " wrong: got " &
                 integer'image(to_integer(unsigned(m_tdata)))
          severity error;
        expected := expected + 1;
      end if;
    end loop;

    report "axis_cdc_tb PASSED" severity note;
    done <= true;
    wait;
  end process;

end architecture;
