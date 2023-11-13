from ortools.sat.python import cp_model
import sys
import time
from datatypes import *
from itertools import combinations


def solve_pbpb(instance):
    model = cp_model.CpModel()

    x = []
    for s in range(instance.k):
        row = []
        for u in range(instance.n):
            if instance.authorisations[u].authorisation_list[s]:
                v = model.NewBoolVar('')
                row.append(v)
            else:
                row.append(None)
        model.Add(sum([v for v in row if v is not None]) == 1)
        x.append(row)

    M = [[None for s1 in range(instance.k)] for s2 in range(instance.k)]
    for s1 in range(instance.k):
        for s2 in range(s1 + 1, instance.k):
            M[s1][s2] = M[s2][s1] = model.NewBoolVar('')

    for (s1, s2) in combinations(range(instance.k), 2):
        for s3 in range(instance.k):
            if s1 == s3 or s2 == s3:
                continue

            model.Add(M[s1][s2] == 1).OnlyEnforceIf([M[s1][s3], M[s2][s3]])
            model.Add(M[s1][s2] == 0).OnlyEnforceIf([M[s2][s3].Not(), M[s1][s3]])
            model.Add(M[s1][s2] == 0).OnlyEnforceIf([M[s2][s3], M[s1][s3].Not()])

    for (s1, s2) in combinations(range(instance.k), 2):
        for u in range(instance.n):
            if x[s1][u] is not None and x[s2][u] is not None:
                model.Add(x[s1][u] == x[s2][u]).OnlyEnforceIf(M[s1][s2])
                model.AddBoolOr([x[s1][u].Not(), x[s2][u].Not()]).OnlyEnforceIf(M[s1][s2].Not())
            if x[s2][u] is None and x[s1][u] is not None:
                model.AddImplication(M[s1][s2], x[s1][u].Not())
            if x[s1][u] is None and x[s2][u] is not None:
                model.AddImplication(M[s1][s2], x[s2][u].Not())

    for c in instance.constraints:
        if isinstance(c, NotEquals):
            model.Add(M[c.s1][c.s2] == 0)
            # model.Add(M[c.s1][c.s2] == 0)
        elif isinstance(c, AtMost):
            for T1 in combinations(c.scope, c.limit + 1):
                model.AddBoolOr([M[s1][s2] for (s1, s2) in combinations(T1, 2)])

        elif isinstance(c, Sual):
            at_least_vars = []
            for T1 in combinations(c.scope, c.limit + 1):
                v = model.NewBoolVar('')
                model.AddBoolAnd([M[s1][s2].Not() for (s1, s2) in combinations(T1, 2)]).OnlyEnforceIf(v)
                at_least_vars.append(v)

            a = model.NewBoolVar('')
            model.AddBoolOr(at_least_vars).OnlyEnforceIf(a)

            for s in c.scope:
                model.AddBoolOr([x[s][u] for u in c.user_group if x[s][u] is not None]).OnlyEnforceIf(a.Not())

        elif isinstance(c, WangLi):
            d = [model.NewBoolVar('') for i in range(len(c.user_groups))]
            for i in range(len(c.user_groups)):
                for s in c.T:
                    model.AddBoolOr([x[s][u] for u in c.user_groups[i] if x[s][u] is not None]).OnlyEnforceIf(d[i])
            model.Add(sum(d) == 1)

        elif isinstance(c, AssignmentDependent):
            a = model.NewBoolVar('')
            model.AddBoolOr([x[c.s1][u] for u in c.U1 if x[c.s1][u] is not None]).OnlyEnforceIf(a)
            model.AddBoolAnd([x[c.s1][u].Not() for u in c.U1 if x[c.s1][u] is not None]).OnlyEnforceIf(a.Not())
            model.AddBoolOr([x[c.s2][u] for u in c.U2 if x[c.s2][u] is not None]).OnlyEnforceIf(a)

        else:
            print('Unknown constraint ' + type(c))
            exit(1)

    solver = cp_model.CpSolver()
    solver.parameters

    start = time.time()
    status = solver.Solve(model)
    end = time.time()

    if status == cp_model.OPTIMAL or status == cp_model.FEASIBLE:
        def get_user(s):
            result = [u for u in range(instance.n) if x[s][u] is not None and solver.Value(x[s][u]) > 0.5]
            assert len(result) == 1
            return result[0]

        return Solution([get_user(s) for s in range(instance.k)], end - start)
    else:
        return Solution(False, end - start)


def main():
    if len(sys.argv) != 3:
        print('Usage: <instance file name> <solution file name>')
        exit(1)

    instance = Instance(sys.argv[1])
    solution = solve_pbpb(instance)
    solution.save(sys.argv[2])
    # assert solution.assignment is False or solution.test_satisfiability(instance)

if __name__ == "__main__":
    main()

