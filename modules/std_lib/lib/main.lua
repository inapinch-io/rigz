function puts(...)
    local args = {...}
    for _, value in ipairs(args) do
        print(value)
    end
end

function var(...)
    local args = {..., n=select("#", ...)}
    for key, value in ipairs(args[args.n]) do
        setfield(key, value)
    end
end

-- Following Functions modified from - https://www.lua.org/pil/14.1.html

function setfield (f, v)
    local t = _G
    for w, d in string.gmatch(f, "([%w_]+)(.?)") do
        if d == "." then
            t[w] = t[w] or {}
            t = t[w]
        else
            t[w] = v
        end
    end
end

function getfield (f)
    local v = _G
    for w in string.gmatch(f, "[%w_]+") do
        v = v[w]
    end
    return v
end