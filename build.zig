const std = @import("std");

pub fn build(b: *std.Build) void {
    // Step: build WASM with wasm-pack
    const wasm_step = b.step("wasm", "Build WASM via wasm-pack");
    const wasm_cmd = b.addSystemCommand(&.{
        "wasm-pack", "build", "--target", "web",
        "--out-dir", "../pkg", "--dev",
    });
    wasm_cmd.setCwd(b.path("rust-wasm"));
    wasm_step.dependOn(&wasm_cmd.step);

    // Step: run the dev server
    const run_step = b.step("run", "Build & run the HTTP server");
    const run_cmd = b.addSystemCommand(&.{
        "cargo", "run", "--package", "serve",
    });
    run_step.dependOn(&run_cmd.step);

    // Default: build wasm then run server
    run_cmd.step.dependOn(wasm_step);
    b.default_step.dependOn(run_step);
}
