# deptree

show dependency tree.

```bash
$ cat input
a:b
b:c
c:d
d:a
$ deptree < input
wrote graph.svg
```

![graph.svg](images/graph.svg)