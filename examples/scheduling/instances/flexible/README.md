# Flexible Jobshop problems

source: https://people.idsia.ch/~monaldo/fjsp.html#ProblemInstances

The naming of the benchmark sets is according to: Schutt, Andreas et al. “Scheduling Optional Tasks with Explanation.”
International Conference on Principles and Practice of Constraint Programming (2013).

## Format

- in the first line there are (at least) 2 numbers: the first is the number of jobs and the second the number of
  machines (the 3rd is not necessary, it is the average number of machines per operation)

- Every row represents one job: the first number is the number of operations of that job, the second number (let's say
  k>
  =1) is the number of machines that can process the first operation; then according to k, there are k pairs of
  numbers (
  machine,processing time) that specify which are the machines and the processing times; then the data for the second
  operation and so on...

Example: Fisher and Thompson 6x6 instance, alternate name (mt06)

```
6   6   1   
6   1   3   1   1   1   3   1   2   6   1   4   7   1   6   3   1   5   6   
6   1   2   8   1   3   5   1   5   10  1   6   10  1   1   10  1   4   4   
6   1   3   5   1   4   4   1   6   8   1   1   9   1   2   1   1   5   7   
6   1   2   5   1   1   5   1   3   5   1   4   3   1   5   8   1   6   9   
6   1   3   9   1   2   3   1   5   5   1   6   4   1   1   3   1   4   1   
6   1   2   3   1   4   3   1   6   9   1   1   10  1   5   4   1   3   1   
```

first row = 6 jobs and 6 machines 1 machine per operation
second row: job 1 has 6 operations, the first operation can be processed by 1 machine that is machine 3 with processing
time 1.
