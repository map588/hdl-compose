// file: selectio_wiz_0_selectio_wiz.v
// (c) Copyright 2017-2018, 2023 Advanced Micro Devices, Inc. All rights reserved.
//
// This file contains confidential and proprietary information
// of AMD and is protected under U.S. and international copyright
// and other intellectual property laws.
//
// DISCLAIMER
// This disclaimer is not a license and does not grant any
// rights to the materials distributed herewith. Except as
// otherwise provided in a valid license issued to you by
// AMD, and to the maximum extent permitted by applicable
// law: (1) THESE MATERIALS ARE MADE AVAILABLE "AS IS" AND
// WITH ALL FAULTS, AND AMD HEREBY DISCLAIMS ALL WARRANTIES
// AND CONDITIONS, EXPRESS, IMPLIED, OR STATUTORY, INCLUDING
// BUT NOT LIMITED TO WARRANTIES OF MERCHANTABILITY, NON-
// INFRINGEMENT, OR FITNESS FOR ANY PARTICULAR PURPOSE; and
// (2) AMD shall not be liable (whether in contract or tort,
// including negligence, or under any other theory of
// liability) for any loss or damage of any kind or nature
// related to, arising under or in connection with these
// materials, including for any direct, or any indirect,
// special, incidental, or consequential loss or damage
// (including loss of data, profits, goodwill, or any type of
// loss or damage suffered as a result of any action brought
// by a third party) even if such damage or loss was
// reasonably foreseeable or AMD had been advised of the
// possibility of the same.
//
// CRITICAL APPLICATIONS
// AMD products are not designed or intended to be fail-
// safe, or for use in any application requiring fail-safe
// performance, such as life-support or safety devices or
// systems, Class III medical devices, nuclear facilities,
// applications related to the deployment of airbags, or any
// other applications that could lead to death, personal
// injury, or severe property or environmental damage
// (individually and collectively, "Critical
// Applications"). Customer assumes the sole risk and
// liability of any use of AMD products in Critical
// Applications, subject only to applicable laws and
// regulations governing limitations on product liability.
//
// THIS COPYRIGHT NOTICE AND DISCLAIMER MUST BE RETAINED AS
// PART OF THIS FILE AT ALL TIMES.
//----------------------------------------------------------------------------
// User entered comments
//----------------------------------------------------------------------------
// None
//----------------------------------------------------------------------------

`timescale 1ps/1ps
module selectio (
    input  data_out_from_device,
    output data_out_to_pins,
    output clk_to_pins,
    input  clk_in,
    input  clk_reset,
    input  io_reset
);

    wire clock_enable = 1'b1;
    wire clk_fwd_out;
    // wire data_out_to_pins_int;
    wire clk_in_int_buf;

    assign clk_in_int_buf = clk_in;

    // Data output buffer
    OBUF #(
        .IOSTANDARD ("LVCMOS33")
    ) obuf_inst_data (
        .O (data_out_to_pins),
        .I (data_out_from_device)
    );


    // Clock forwarding with SAME_EDGE
    ODDR #(
        .DDR_CLK_EDGE ("SAME_EDGE"),
        .INIT         (1'b0),
        .SRTYPE       ("ASYNC")
    ) oddr_inst_clk (
        .D1  (1'b1),
        .D2  (1'b0),
        .C   (clk_in_int_buf),
        .CE  (clock_enable),
        .Q   (clk_fwd_out),
        .R   (clk_reset),
        .S   (1'b0)
    );

    // Clock output buffer
     (* IOB = "true" *)
    OBUF #(
        .IOSTANDARD ("LVCMOS33")
    ) obuf_inst_clk (
        .O (clk_to_pins),
        .I (clk_fwd_out)
    );

endmodule
