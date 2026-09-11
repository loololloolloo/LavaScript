-- LavaScript extended control flow
const start = 1

for i = start, 8
    if i == 3 then
        continue
    end
    if i == 7 then
        break
    end
    print i
end

let n = 0
repeat
    n = n + 1
until n >= 3

do
    print n
end
