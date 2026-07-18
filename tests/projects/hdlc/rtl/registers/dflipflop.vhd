LIBRARY ieee;
USE ieee.std_logic_1164.all;
USE ieee.numeric_std.all;

ENTITY d_ff IS
   PORT( 
      CLK : IN     std_logic;
      D   : IN     std_logic;
      EN  : IN     std_logic;
      RST : IN     std_logic;
      Q   : OUT    std_logic
   );

END d_ff;


ARCHITECTURE flipflop OF d_ff IS
BEGIN
     CLKD : process(CLK, RST)
     begin
        if(RST = '1') then
           Q <= '0';
        elsif(CLK'event AND CLK = '1') then
           if(EN = '1') then
              Q <= D;
           end if;
        end if;
     end process CLKD;
END flipflop;