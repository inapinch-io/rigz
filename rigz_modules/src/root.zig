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
    error_message: ?*const c_char
};

pub export fn initialize_module(runtime: *ModuleRuntime, module: Module) RuntimeStatus {
    _ = runtime;
    _ = module;
    // call module's initialize function if it exists
    return RuntimeStatus{.status = 0, .value = .{ .tag = core.None }, .error_message = null};
}

var global_runtime: ModuleRuntime = ModuleRuntime{};

pub export fn module_runtime() ModuleRuntime {
    return global_runtime;
}

pub export fn invoke_symbol(name: core.StrSlice, arguments: core.ArgumentVector, definition: core.ArgumentDefinition, prior_result: *core.Argument) RuntimeStatus {
    _ = name;
    _ = arguments;
    _ = definition;
    _ = prior_result;
    return RuntimeStatus{.status = 0, .value = .{ .tag = core.None }, .error_message = null};
}
