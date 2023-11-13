from ortools.sat.python import cp_model
import sys
import time
from datatypes import *


def solve_udpb(instance):
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

    for c in instance.constraints:
        if isinstance(c, NotEquals):
            for u in range(instance.n):
                if x[c.s1][u] is not None and x[c.s2][u] is not None:
                    model.AddBoolOr([x[c.s1][u].Not(), x[c.s2][u].Not()])

        elif isinstance(c, AtMost):
            z = [model.NewBoolVar('') for u in range(instance.n)]
            for u in range(instance.n):
                for s in c.scope:
                    if x[s][u] is not None:
                        model.AddImplication(x[s][u], z[u])

            model.Add(sum(z) <= c.limit)

        elif isinstance(c, WangLi):
            d = [model.NewBoolVar('') for i in range(len(c.Teams))]
            for i in range(len(c.Teams)):
                for u in c.Teams[i]:
                    for s in c.T:
                        if x[s][u] is not None:
                            model.AddImplication(d[i].Not(), x[s][u].Not())

            model.AddBoolOr(d)

            # Restrict users not in the teams to be assigned the steps T
            for u in range(instance.n):
                if not any(u in team for team in c.Teams):
                    for s in c.T:
                        model.Add(x[s][u] == 0)


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
            for u in range(instance.n):
                if x[s][u] is not None and solver.Value(x[s][u]) > 0.5:
                    return u
            raise Exception()

        return Solution([get_user(s) for s in range(instance.k)], end - start)
    else:
        return Solution(False, end - start)


def main():
    if len(sys.argv) != 3:
        print('Usage: <instance file name> <solution file name>')
        exit(1)

    instance = Instance(sys.argv[1])
    solution = solve_udpb(instance)
    solution.save(sys.argv[2])


# This block ensures that main() is not run when this script is imported as a module.
if __name__ == "__main__":
    main()