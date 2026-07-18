library IEEE;
  use IEEE.STD_LOGIC_1164.all;

entity bitwise_vec_op is
  generic (
    OP   : string   := "AND";
    SIZE : positive := 8
  );
  port (
    input_vector : in  std_logic_vector(SIZE - 1 downto 0);
    result   : out std_logic
  );
end entity;

architecture RTL of bitwise_vec_op is
begin
  -- Generate the appropriate reduction operation based on OP
  op_select: process (input_vector)
    variable temp : std_logic;
  begin
    -- if/elsif instead of case: case choices on a string generic must all
    -- have equal length, which "OR"/"NAND" violate
    if OP = "AND" then
      temp := '1';
      for i in input_vector'range loop
        temp := temp and input_vector(i);
      end loop;

    elsif OP = "OR" then
      temp := '0';
      for i in input_vector'range loop
        temp := temp or input_vector(i);
      end loop;

    elsif OP = "XOR" then
      temp := '0';
      for i in input_vector'range loop
        temp := temp xor input_vector(i);
      end loop;

    elsif OP = "NAND" then
      temp := '1';
      for i in input_vector'range loop
        temp := temp and input_vector(i);
      end loop;
      temp := not temp;

    elsif OP = "NOR" then
      temp := '0';
      for i in input_vector'range loop
        temp := temp or input_vector(i);
      end loop;
      temp := not temp;

    else
      temp := '0';
    end if;

    result <= temp;
  end process;
end architecture;
