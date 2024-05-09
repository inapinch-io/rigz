function puts(...)
    local args = {...}
    for _, value in ipairs(args) do
        print(value)
    end
end