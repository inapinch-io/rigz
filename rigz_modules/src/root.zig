const std = @import("std");
const core = @cImport({
    @cInclude("rigz_core.h");
});
const c = @cImport({
    @cInclude("dlfcn.h");
});

const testing = std.testing;

const ModuleFunctionType = fn(
    name: core.StrSlice,
    arguments: core.ArgumentVector,
    definition: core.ArgumentDefinition,
    prior_result: *core.Argument
) RuntimeStatus;

const DynamicLibrary = struct {
    handle: ?*c.void = null,

    pub fn open(path: []const u8) !*DynamicLibrary {
        const lib_path = std.mem.span(path);
        const handle = c.dlopen(lib_path.ptr, c.RTLD_LAZY);
        if (handle == null) return error.LibraryNotFound;
        return &DynamicLibrary{ .handle = handle };
    }

    pub fn loadFn(self: *DynamicLibrary, fn_name: []const u8) !*const ModuleFunctionType {
        const name = std.mem.span(fn_name);
        const func_ptr = c.dlsym(self.handle, name.ptr);
        if (func_ptr == null) return error.FunctionNotFound;
        return @ptrCast(func_ptr);
    }

    pub fn close(self: *DynamicLibrary) void {
        if (self.handle != null) {
            _ = c.dlclose(self.handle);
        }
    }
};

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
