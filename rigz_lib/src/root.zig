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
}

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "basic add functionality" {
    try testing.expect(add(3, 7) == 10);
}
