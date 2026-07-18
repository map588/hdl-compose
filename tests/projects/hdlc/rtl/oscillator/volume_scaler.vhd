library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity volume_scaler is
    port (
        -- Control signals
        clk         : in  std_logic;
        rstn        : in  std_logic;
        en          : in  std_logic;  -- Mute control
        -- Input ports
        s_axis_sample_tdata   : in  std_logic_vector(23 downto 0);  -- Audio sample input
        s_axis_sample_tvalid    : in  std_logic;                      -- Input data valid
		velocity: in std_logic_vector(6 downto 0); -- Velocity (0-127)

        -- Output ports
        m_axis_data_tdata    : out std_logic_vector(23 downto 0);  -- Scaled output
        m_axis_data_tvalid   : out std_logic                       -- Output data valid
    );

end entity volume_scaler;

architecture rtl of volume_scaler is    
    -- State types for each pipeline stage
    type stage_state is (IDLE, LOAD, SCALE, DONE);

    
    -- State signals for each stage
    signal s1 : stage_state;  -- Input stage
    signal s2 : stage_state;  -- Multiply stage
    signal s3 : stage_state;  -- Scale stage
    
    -- Pipeline stage data registers
    signal stage1_data : std_logic_vector(23 downto 0);
    signal stage3_data : std_logic_vector(23 downto 0);
    
    -- Pipeline stage done flags
    signal stage1_done : std_logic;
    signal stage2_done : std_logic;
    signal stage3_done : std_logic;

        -- Internal signals
    signal sample_signed     : signed(23 downto 0);
    signal velocity_unsigned : unsigned(6 downto 0);
    
    -- Additional registers
    signal velocity_reg      : std_logic_vector(6 downto 0);
    signal product_reg      : signed(35 downto 0);
    
	-- DSP48E1 attributes for multiplication pipelining
	attribute use_dsp                : string;
	attribute use_dsp of product_reg : signal is "yes";

    -- Input A pipeline registers and cascade registers
	attribute AREG                    : integer;
	attribute ACASCREG                : integer;
	attribute AREG     of product_reg : signal is 1;
	attribute ACASCREG of product_reg : signal is 1;

    -- Input B pipeline registers and cascade registers
	attribute BREG                    : integer;
	attribute BCASCREG                : integer;
	attribute BREG     of product_reg : signal is 1;
	attribute BCASCREG of product_reg : signal is 1;

    -- Multiplier output register
	attribute MREG                : integer;
	attribute MREG of product_reg : signal is 1;

    -- DSP output register
	attribute PREG                : integer;
	attribute PREG of product_reg : signal is 1;


    
begin
    -- Convert inputs to signed/unsigned
    sample_signed     <= signed(stage1_data);
    velocity_unsigned <= unsigned(velocity_reg);
    
    -- Stage 1: Input Process
    process(clk, rstn)
    begin
        if rising_edge(clk) then
          if rstn = '0' then
            s1           <= IDLE;
            stage1_data  <= (others => '0');
            stage1_done  <= '0';
            velocity_reg <= (others => '0');
        else
            case s1 is
                when IDLE =>
                    if s_axis_sample_tvalid = '1' then
                        s1 <= LOAD;
                        stage1_data <= s_axis_sample_tdata;
                        velocity_reg <= velocity;
                        stage1_done <= '1';
                    else
                        stage1_done <= '0';
                    end if;
                    
                when LOAD =>
                    if s_axis_sample_tvalid = '1' then
                        stage1_data <= s_axis_sample_tdata;
                        velocity_reg <= velocity;
                        stage1_done <= '1';
                    else
                        stage1_done <= '0';
                        s1 <= IDLE;
                    end if;
                    
                when others =>
                    s1 <= IDLE;
            end case;
        end if;
        end if;
    end process;
    
    -- Stage 2: Multiply Process
    process(clk, rstn)
        variable temp_product : signed(35 downto 0);
    begin
        if rising_edge(clk) then
          if rstn = '0' then
            s2          <= IDLE;
            product_reg <= (others => '0');
            stage2_done <= '0';
          else
            case s2 is
                when IDLE =>
                    if stage1_done = '1' then
                        s2 <= SCALE;
                        temp_product := sample_signed * signed('0' & velocity_unsigned & "1111");
                        product_reg <= temp_product;
                        stage2_done <= '1';
                    else
                        stage2_done <= '0';
                    end if;
                    
                when SCALE =>
                    if stage1_done = '1' then
                        temp_product := sample_signed * signed('0' & velocity_unsigned & "1111");
                        product_reg <= temp_product;
                        stage2_done <= '1';
                    else
                        stage2_done <= '0';
                        s2 <= IDLE;
                    end if;
                    
                when others =>
                    s2 <= IDLE;
            end case;
        end if;
    end if;
    end process;
    
    -- Stage 3: Scale Process
    process(clk, rstn)
    begin
        if rising_edge(clk) then
          if rstn = '0' then
            s3          <= IDLE;
            stage3_data <= (others => '0');
            stage3_done <= '0';
          else
            case s3 is
                when IDLE =>
                    if stage2_done = '1' then
                        s3 <= SCALE;
                        stage3_data <= std_logic_vector(product_reg(35 downto 12));
                        stage3_done <= '1';
                    else
                        stage3_done <= '0';
                    end if;
                    
                when SCALE =>
                    if stage2_done = '1' then
                        stage3_data <= std_logic_vector(product_reg(35 downto 12));
                        stage3_done <= '1';
                    else
                        stage3_done <= '0';
                        s3 <= IDLE;
                    end if;
                    
                when others =>
                    s3 <= IDLE;
            end case;
        end if;
    end if;
    end process;
    
    -- Output assignments
    m_axis_data_tdata <= stage3_data when en = '1' else (others => '0');
    m_axis_data_tvalid <= stage3_done when en = '1' else '0';
    
end rtl;