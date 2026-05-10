module fifo_sync #(
    parameter DEPTH = 256,
    parameter WIDTH = 8
) (
    input  wire             clk,
    input  wire             rst_n,
    input  wire [WIDTH-1:0] din,
    input  wire             wr_en,
    input  wire             rd_en,
    output wire [WIDTH-1:0] dout,
    output wire             full,
    output wire             empty
);

// stub
assign dout  = {WIDTH{1'b0}};
assign full  = 1'b0;
assign empty = 1'b1;

endmodule
