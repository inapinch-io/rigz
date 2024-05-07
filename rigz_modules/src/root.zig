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
    handle: ?*anyopaque = null,

    pub fn open(path: Str) !*DynamicLibrary {
        const handle = c.dlopen(path.ptr, c.RTLD_LAZY);
        if (handle == null) return error.LibraryNotFound;
        var lib = DynamicLibrary{ .handle = handle };
        return &lib;
    }

    pub fn loadFixedFn(self: *DynamicLibrary, fn_name: Str) !*const ModuleFixedFunctionType {
        const func_ptr = c.dlsym(self.handle, fn_name.ptr);
        if (func_ptr == null) return error.FunctionNotFound;
        return @ptrCast(func_ptr);
    }

    pub fn loadPassthroughFn(self: *DynamicLibrary, fn_name: Str) !*const ModulePassFunctionType {
        const func_ptr = c.dlsym(self.handle, fn_name.ptr);
        if (func_ptr == null) return error.FunctionNotFound;
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
    const dynamic = DynamicLibrary.open(library_path) catch {
        return ModuleStatus{
            .status = 1,
            .value = .{
                .format = 0,
                .name = name,
                .handle = null,
                .pass_through = null
            },
            .error_message = @ptrCast("Library Not Found")
        };
    };

    const library = .{ .format = 0, .name = name, .path = library_path, .handle = dynamic.handle, .pass_through = null };
    return ModuleStatus{.status = 0, .value = library, .error_message = null};
}

pub export fn invoke_symbol(library: core.Library, name: Str, arguments: core.ArgumentVector, definition: core.ArgumentDefinition, prior_result: *core.Argument) core.RuntimeStatus {
    const lib = DynamicLibrary.open(library.path) catch {
        return core.RuntimeStatus{.status = 1, .value = .{ .tag = core.None }, .error_message = "Library Not Found: Invoke"};
    };

    return switch (library.format) {
        0 => {
            const func = lib.loadFixedFn(name) catch {
                return core.RuntimeStatus{.status = 1, .value = .{ .tag = core.None }, .error_message = "Function Invocation Failed"};
            };
            std.debug.print("Calling Function\n", .{});
            return func(arguments, definition, prior_result);
        },
        1 => {
            std.debug.print("Calling Function\n", .{});
            const func = lib.loadPassthroughFn(name) catch {
                return core.RuntimeStatus{.status = 1, .value = .{ .tag = core.None }, .error_message = "Function Invocation Failed"};
            };
            return func(name, arguments, definition, prior_result);
        },
        else => {
            std.debug.print("Invalid Format\n", .{});
            return core.RuntimeStatus{.status = 1, .value = .{ .tag = core.None }, .error_message = "Unsupported Function Format"};
        }
    };
}
