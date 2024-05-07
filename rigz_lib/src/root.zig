const std = @import("std");
const testing = std.testing;
const core = @cImport({
    @cInclude("rigz_core.h");
});

export fn puts(
    arguments: core.ArgumentVector,
    definition: core.ArgumentDefinition,
    prior_result: *core.Argument
) RuntimeStatus {
    const slice = core.arguments_to_str(arguments);
    const zig_slice = @ptrCast([*]const u8, slice.ptr)[0..slice.len];
    std.debug.print("{}\n", .{zig_slice});
    return core.default_runtime_response();
}