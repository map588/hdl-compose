----------------------------------------------------------------------------------
-- Company: 
-- Engineer: 
-- 
-- Create Date: 11/30/2024 07:15:23 PM
-- Design Name: 
-- Module Name: output_stage - Behavioral
-- Project Name: 
-- Target Devices: 
-- Tool Versions: 
-- Description: 
-- 
-- Dependencies: 
-- 
-- Revision:
-- Revision 0.01 - File Created
-- Additional Comments:
-- 
----------------------------------------------------------------------------------

library IEEE;
  use IEEE.STD_LOGIC_1164.all;

  -- Uncomment the following library declaration if using
  -- arithmetic functions with Signed or Unsigned values
  --use IEEE.NUMERIC_STD.ALL;
  -- Uncomment the following library declaration if instantiating
  -- any Xilinx leaf cells in this code.
  --library UNISIM;
  --use UNISIM.VComponents.all;

entity output_stage is
  port (
    -- Square input
    s_axis_audio_tdata  : in  std_logic_vector(23 downto 0);
    s_axis_audio_tvalid : in  std_logic;
    s_axis_audio_tready : out std_logic;

    -- Output
    m_axis_out_tdata    : out std_logic_vector(23 downto 0);
    m_axis_out_tvalid   : out std_logic;
    m_axis_out_tready   : in  std_logic;

    clk                : in  std_logic;
    enable             : in  std_logic

  );
end entity;

architecture RTL of output_stage is

begin

    process (clk) is
    begin
        if rising_edge(clk) then
            if enable = '1' then
              s_axis_audio_tready <= m_axis_out_tready;
              if s_axis_audio_tvalid = '1' and m_axis_out_tready = '1' then
                m_axis_out_tdata <= s_axis_audio_tdata;
                m_axis_out_tvalid <= '1';
            elsif m_axis_out_tready = '1' then
              m_axis_out_tvalid <= '0';
            end if;
          end if;
          end if;
    end process;

end architecture;
