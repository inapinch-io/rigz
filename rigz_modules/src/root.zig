const std = @import("std");
const core = @cImport({
    @cInclude("rigz_core.h");
});
const c = @cImport({
    @cInclude("dlfcn.h");
});

const testing = std.testing;

const Str = core.StrSlice;

const ModuleFixedFunctionType = fn(
    arguments: core.ArgumentVector,
    definition: core.ArgumentDefinition,
    prior_result: *core.Argument
) core.RuntimeStatus;

const ModulePassFunctionType = fn(
    name: core.StrSlice,
    arguments: core.ArgumentVector,
    definition: core.ArgumentDefinition,
    prior_result: *core.Argument
) core.RuntimeStatus;

const DynamicLibrary = struct {
    handle: ?*c.void = null,

    pub fn open(path: []const u8) !*DynamicLibrary {
        const lib_path = std.mem.span(path);
        const handle = c.dlopen(lib_path.ptr, c.RTLD_LAZY);
        if (handle == null) return error.LibraryNotFound;
        return &DynamicLibrary{ .handle = handle };
    }

    fn loadFn(self: *DynamicLibrary, fn_name: []const u8) !*const ModuleFixedFunctionType {
        const name = std.mem.span(fn_name);
        const func_ptr = c.dlsym(self.handle, name.ptr);
        if (func_ptr == null) return error.FunctionNotFound;
        return func_ptr;
    }

    pub fn loadFixedFn(self: *DynamicLibrary, fn_name: []const u8) !*const ModuleFixedFunctionType {
        const func_ptr = self.loadFn(fn_name);
        return @ptrCast(func_ptr);
    }

    pub fn loadPassthroughFn(self: *DynamicLibrary, fn_name: []const u8) !*const ModulePassFunctionType {
        const func_ptr = self.loadFn(fn_name);
        return @ptrCast(func_ptr);
    }

    pub fn close(self: *DynamicLibrary) void {
        if (self.handle != null) {
            _ = c.dlclose(self.handle);
        }
    }
};

pub const ModuleStatus = extern struct {
    status: c_int,
    value: core.Library,
    error_message: ?*const c_char
};

pub export fn initialize_module(name: Str, library_path: Str) ModuleStatus {
    _ = library_path;
    // call module's initialize function if it exists
    return ModuleStatus{.status = 0, .value = .{ .format = 0, .name = name, .handle = null, .pass_through = null }, .error_message = null};
}

pub export fn invoke_symbol(library: core.Library, name: Str, arguments: core.ArgumentVector, definition: core.ArgumentDefinition, prior_result: *core.Argument) core.RuntimeStatus {
    _ = library;
    _ = name;
    _ = arguments;
    _ = definition;
    _ = prior_result;
    return core.RuntimeStatus{.status = 0, .value = .{ .tag = core.None }, .error_message = null};
}
