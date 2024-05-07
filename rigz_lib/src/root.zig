const std = @import("std");
const testing = std.testing;
const core = @cImport({
    @cInclude("rigz_core.h");
});

pub export fn puts(
    arguments: core.ArgumentVector,
    definition: core.ArgumentDefinition,
    prior_result: *core.Argument
) core.RuntimeStatus {
    _ = definition;
    _ = prior_result;
    const slice = core.arguments_to_str(arguments);
    const zig_slice: [*]const u8= @ptrCast(slice.ptr);
    std.debug.print("{s}\n", .{zig_slice[0..slice.len]});
    return core.default_runtime_response();
}