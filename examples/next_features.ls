-- LavaScript feature showcase

function classify(x)
    if x < 0 then
        return -1
    elseif x == 0 then
        return 0
    else
        return 1
    end
end

let value = 7
let enabled = true
let result = classify(value)

print "LavaScript feature test"
print result
print enabled and not false
print value % 3

while enabled
    value = value - 1
    if value == 2 || value == 1 then
        print value
    end
    if value == 0 then
        break
    end
end

print "done"
