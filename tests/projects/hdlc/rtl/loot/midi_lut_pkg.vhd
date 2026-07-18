library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;


package midi_lut_pkg is
    type phase_inc_lut_type is array (0 to 127) of std_logic_vector(23 downto 0);
    constant midi_phase_inc_lut : phase_inc_lut_type;
end package midi_lut_pkg;

package body midi_lut_pkg is
    constant midi_phase_inc_lut: phase_inc_lut_type := (
        X"000B29",  -- MIDI Note   0,  Freq  8.1758 Hz
        X"000BD3",  -- MIDI Note   1,  Freq  8.6620 Hz
        X"000C87",  -- MIDI Note   2,  Freq  9.1770 Hz
        X"000D46",  -- MIDI Note   3,  Freq  9.7227 Hz
        X"000E10",  -- MIDI Note   4,  Freq  10.3009 Hz
        X"000EE6",  -- MIDI Note   5,  Freq  10.9134 Hz
        X"000FC9",  -- MIDI Note   6,  Freq  11.5623 Hz
        X"0010B9",  -- MIDI Note   7,  Freq  12.2499 Hz
        X"0011B8",  -- MIDI Note   8,  Freq  12.9783 Hz
        X"0012C5",  -- MIDI Note   9,  Freq  13.7500 Hz
        X"0013E3",  -- MIDI Note  10,  Freq  14.5676 Hz
        X"001512",  -- MIDI Note  11,  Freq  15.4339 Hz
        X"001653",  -- MIDI Note  12,  Freq  16.3516 Hz
        X"0017A7",  -- MIDI Note  13,  Freq  17.3239 Hz
        X"00190F",  -- MIDI Note  14,  Freq  18.3540 Hz
        X"001A8C",  -- MIDI Note  15,  Freq  19.4454 Hz
        X"001C20",  -- MIDI Note  16,  Freq  20.6017 Hz
        X"001DCD",  -- MIDI Note  17,  Freq  21.8268 Hz
        X"001F92",  -- MIDI Note  18,  Freq  23.1247 Hz
        X"002173",  -- MIDI Note  19,  Freq  24.4997 Hz
        X"002370",  -- MIDI Note  20,  Freq  25.9565 Hz
        X"00258B",  -- MIDI Note  21,  Freq  27.5000 Hz
        X"0027C7",  -- MIDI Note  22,  Freq  29.1352 Hz
        X"002A25",  -- MIDI Note  23,  Freq  30.8677 Hz
        X"002CA6",  -- MIDI Note  24,  Freq  32.7032 Hz
        X"002F4E",  -- MIDI Note  25,  Freq  34.6478 Hz
        X"00321E",  -- MIDI Note  26,  Freq  36.7081 Hz
        X"003519",  -- MIDI Note  27,  Freq  38.8909 Hz
        X"003841",  -- MIDI Note  28,  Freq  41.2034 Hz
        X"003B9A",  -- MIDI Note  29,  Freq  43.6535 Hz
        X"003F25",  -- MIDI Note  30,  Freq  46.2493 Hz
        X"0042E6",  -- MIDI Note  31,  Freq  48.9994 Hz
        X"0046E0",  -- MIDI Note  32,  Freq  51.9131 Hz
        X"004B17",  -- MIDI Note  33,  Freq  55.0000 Hz
        X"004F8F",  -- MIDI Note  34,  Freq  58.2705 Hz
        X"00544A",  -- MIDI Note  35,  Freq  61.7354 Hz
        X"00594D",  -- MIDI Note  36,  Freq  65.4064 Hz
        X"005E9C",  -- MIDI Note  37,  Freq  69.2957 Hz
        X"00643C",  -- MIDI Note  38,  Freq  73.4162 Hz
        X"006A32",  -- MIDI Note  39,  Freq  77.7817 Hz
        X"007083",  -- MIDI Note  40,  Freq  82.4069 Hz
        X"007734",  -- MIDI Note  41,  Freq  87.3071 Hz
        X"007E4A",  -- MIDI Note  42,  Freq  92.4986 Hz
        X"0085CD",  -- MIDI Note  43,  Freq  97.9989 Hz
        X"008DC1",  -- MIDI Note  44,  Freq  103.8262 Hz
        X"00962F",  -- MIDI Note  45,  Freq  110.0000 Hz
        X"009F1E",  -- MIDI Note  46,  Freq  116.5409 Hz
        X"00A894",  -- MIDI Note  47,  Freq  123.4708 Hz
        X"00B29A",  -- MIDI Note  48,  Freq  130.8128 Hz
        X"00BD39",  -- MIDI Note  49,  Freq  138.5913 Hz
        X"00C879",  -- MIDI Note  50,  Freq  146.8324 Hz
        X"00D465",  -- MIDI Note  51,  Freq  155.5635 Hz
        X"00E106",  -- MIDI Note  52,  Freq  164.8138 Hz
        X"00EE68",  -- MIDI Note  53,  Freq  174.6141 Hz
        X"00FC95",  -- MIDI Note  54,  Freq  184.9972 Hz
        X"010B9A",  -- MIDI Note  55,  Freq  195.9977 Hz
        X"011B83",  -- MIDI Note  56,  Freq  207.6523 Hz
        X"012C5F",  -- MIDI Note  57,  Freq  220.0000 Hz
        X"013E3C",  -- MIDI Note  58,  Freq  233.0819 Hz
        X"015128",  -- MIDI Note  59,  Freq  246.9417 Hz
        X"016534",  -- MIDI Note  60,  Freq  261.6256 Hz
        X"017A72",  -- MIDI Note  61,  Freq  277.1826 Hz
        X"0190F3",  -- MIDI Note  62,  Freq  293.6648 Hz
        X"01A8CA",  -- MIDI Note  63,  Freq  311.1270 Hz
        X"01C20D",  -- MIDI Note  64,  Freq  329.6276 Hz
        X"01DCD0",  -- MIDI Note  65,  Freq  349.2282 Hz
        X"01F92A",  -- MIDI Note  66,  Freq  369.9944 Hz
        X"021734",  -- MIDI Note  67,  Freq  391.9954 Hz
        X"023707",  -- MIDI Note  68,  Freq  415.3047 Hz
        X"0258BF",  -- MIDI Note  69,  Freq  440.0000 Hz
        X"027C78",  -- MIDI Note  70,  Freq  466.1638 Hz
        X"02A250",  -- MIDI Note  71,  Freq  493.8833 Hz
        X"02CA69",  -- MIDI Note  72,  Freq  523.2511 Hz
        X"02F4E4",  -- MIDI Note  73,  Freq  554.3653 Hz
        X"0321E6",  -- MIDI Note  74,  Freq  587.3295 Hz
        X"035195",  -- MIDI Note  75,  Freq  622.2540 Hz
        X"03841A",  -- MIDI Note  76,  Freq  659.2551 Hz
        X"03B9A0",  -- MIDI Note  77,  Freq  698.4565 Hz
        X"03F254",  -- MIDI Note  78,  Freq  739.9888 Hz
        X"042E68",  -- MIDI Note  79,  Freq  783.9909 Hz
        X"046E0F",  -- MIDI Note  80,  Freq  830.6094 Hz
        X"04B17E",  -- MIDI Note  81,  Freq  880.0000 Hz
        X"04F8F0",  -- MIDI Note  82,  Freq  932.3275 Hz
        X"0544A1",  -- MIDI Note  83,  Freq  987.7666 Hz
        X"0594D3",  -- MIDI Note  84,  Freq  1046.5023 Hz
        X"05E9C9",  -- MIDI Note  85,  Freq  1108.7305 Hz
        X"0643CD",  -- MIDI Note  86,  Freq  1174.6591 Hz
        X"06A32B",  -- MIDI Note  87,  Freq  1244.5079 Hz
        X"070834",  -- MIDI Note  88,  Freq  1318.5102 Hz
        X"077340",  -- MIDI Note  89,  Freq  1396.9129 Hz
        X"07E4A9",  -- MIDI Note  90,  Freq  1479.9777 Hz
        X"085CD1",  -- MIDI Note  91,  Freq  1567.9817 Hz
        X"08DC1E",  -- MIDI Note  92,  Freq  1661.2188 Hz
        X"0962FC",  -- MIDI Note  93,  Freq  1760.0000 Hz
        X"09F1E0",  -- MIDI Note  94,  Freq  1864.6550 Hz
        X"0A8942",  -- MIDI Note  95,  Freq  1975.5332 Hz
        X"0B29A6",  -- MIDI Note  96,  Freq  2093.0045 Hz
        X"0BD392",  -- MIDI Note  97,  Freq  2217.4610 Hz
        X"0C879A",  -- MIDI Note  98,  Freq  2349.3181 Hz
        X"0D4656",  -- MIDI Note  99,  Freq  2489.0159 Hz
        X"0E1069",  -- MIDI Note 100,  Freq  2637.0205 Hz
        X"0EE680",  -- MIDI Note 101,  Freq  2793.8259 Hz
        X"0FC953",  -- MIDI Note 102,  Freq  2959.9554 Hz
        X"10B9A2",  -- MIDI Note 103,  Freq  3135.9635 Hz
        X"11B83C",  -- MIDI Note 104,  Freq  3322.4376 Hz
        X"12C5F9",  -- MIDI Note 105,  Freq  3520.0000 Hz
        X"13E3C0",  -- MIDI Note 106,  Freq  3729.3101 Hz
        X"151285",  -- MIDI Note 107,  Freq  3951.0664 Hz
        X"16534C",  -- MIDI Note 108,  Freq  4186.0090 Hz
        X"17A725",  -- MIDI Note 109,  Freq  4434.9221 Hz
        X"190F34",  -- MIDI Note 110,  Freq  4698.6363 Hz
        X"1A8CAC",  -- MIDI Note 111,  Freq  4978.0317 Hz
        X"1C20D2",  -- MIDI Note 112,  Freq  5274.0409 Hz
        X"1DCD01",  -- MIDI Note 113,  Freq  5587.6517 Hz
        X"1F92A6",  -- MIDI Note 114,  Freq  5919.9108 Hz
        X"217345",  -- MIDI Note 115,  Freq  6271.9270 Hz
        X"237078",  -- MIDI Note 116,  Freq  6644.8752 Hz
        X"258BF2",  -- MIDI Note 117,  Freq  7040.0000 Hz
        X"27C780",  -- MIDI Note 118,  Freq  7458.6202 Hz
        X"2A250B",  -- MIDI Note 119,  Freq  7902.1328 Hz
        X"2CA698",  -- MIDI Note 120,  Freq  8372.0181 Hz
        X"2F4E4B",  -- MIDI Note 121,  Freq  8869.8442 Hz
        X"321E68",  -- MIDI Note 122,  Freq  9397.2726 Hz
        X"351958",  -- MIDI Note 123,  Freq  9956.0635 Hz
        X"3841A5",  -- MIDI Note 124,  Freq  10548.0818 Hz
        X"3B9A03",  -- MIDI Note 125,  Freq  11175.3034 Hz
        X"3F254D",  -- MIDI Note 126,  Freq  11839.8215 Hz
        X"42E68A"   -- MIDI Note 127,  Freq  12543.8540 Hz
    );
end package body midi_lut_pkg;
