const std = @import("std");
const core = @cImport({
    @cInclude("rigz_core.h");
});
const testing = std.testing;

pub const ModuleRuntime = extern struct {

};

// Matching Rust's `Module` struct
pub const Module = extern struct {
    name: core.StrSlice,
    library_path: core.StrSlice,
};

pub const RuntimeStatus = extern struct {
    status: c_int,
    value: core.Argument,
};

pub export fn initialize_module(runtime: *ModuleRuntime, module: Module) RuntimeStatus {
    _ = runtime;
    _ = module;
    // call module's initialize function if it exists
    return RuntimeStatus{.status = 0, .value = .{ .tag = core.None }};
}

var global_runtime: ModuleRuntime = ModuleRuntime{};

pub export fn module_runtime() ModuleRuntime {
    return global_runtime;
}

pub export fn invoke_symbol(name: core.StrSlice) RuntimeStatus {
    _ = name;
    return RuntimeStatus{.status = 0, .value = .{ .tag = core.None }};
}
