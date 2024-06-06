import os

ls = []
for (dirpath, dirnames, filenames) in os.walk(".\\src"):
    for f in filenames:
        if f.endswith(".rs"):
            name = dirpath + "\\" + f
            with open(name, "r") as file:
                count = len(file.readlines())
            ls.append((name, count))

total = 0
total_test = 0
for (n, c) in sorted(ls, key=lambda x: x[1]):
    print(n, c)
    if "\\tests" in n:
        total_test += c
    else:
        total += c

print("total:", total)
print("total test:", total_test)

