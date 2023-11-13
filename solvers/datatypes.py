import re


def check_line(f, expected_str):
    line = f.readline()
    if line.strip().lower() != expected_str:
        raise Exception(f'"{expected_str}" expected but "{line}" found.')


class Instance:
    def __init__(self, filename):
        f = open(filename, 'r')

        self.k = int(re.match(r'#Steps:\s+(\d+)', f.readline(), re.IGNORECASE).group(1))
        self.n = int(re.match(r'#Users:\s+(\d+)', f.readline(), re.IGNORECASE).group(1))
        num_of_constraints = int(re.match(r'#Constraints:\s+(\d+)', f.readline(), re.IGNORECASE).group(1))

        check_line(f, "authorizations:")

        self.authorisations = []
        for u in range(self.n):
            a = Authorisations()
            line = f.readline()
            a.read(line)
            if not a.test_feasibility(self):
                raise Exception(f'Mistake in authorisations "{line}".')

            assert a.u == u
            self.authorisations.append(a)

        check_line(f, "constraints:")

        self.constraints = []
        for i in range(num_of_constraints):
            line = f.readline().lower()
            if line.startswith('sod'):
                c = NotEquals()
            elif line.startswith('at most'):
                c = AtMost()
            elif line.startswith('sual scope'):
                c = Sual()
            elif line.startswith('wang-li'):
                c = WangLi()
            elif line.startswith('assignment-dependent scope'):
                c = AssignmentDependent()
            else:
                raise Exception(f'Unknown constraint: {line}')

            c.read(line)
            if not c.test_feasibility(self):
                raise Exception(f'Mistake in constraint "{line}".')

            self.constraints.append(c)

        f.close()


class Authorisations:
    def __init__(self):
        self.u = -1
        self.authorisation_list = []

    def read(self, line):
        match = re.match(r'user\s+(\d+):(.+)$', line, re.IGNORECASE)
        self.u = int(match.group(1)) - 1
        matches = re.findall(r'\s+(\d+)', match.group(2), re.IGNORECASE)
        self.authorisation_list = [int(m) for m in matches]

    def test_feasibility(self, instance):
        return 0 <= self.u < instance.n


class NotEquals:
    def __init__(self):
        self.s1 = -1
        self.s2 = -1

    def read(self, line):
        match = re.match(r'sod scope\s+(\d+)\s+(\d+)', line, re.IGNORECASE)
        self.s1 = int(match.group(1)) - 1
        self.s2 = int(match.group(2)) - 1

    def test_feasibility(self, instance):
        return 0 <= self.s1 < instance.k \
               and 0 <= self.s2 < instance.k

    def test_satisfiability(self, solution):
        return solution.assignment[self.s1] != solution.assignment[self.s2]


class AtMost:
    def __init__(self):
        self.limit = -1
        self.scope = []

    def read(self, line):
        match = re.match(r'at most\s+(\d+)\s+scope\s+([\d\s]+)', line, re.IGNORECASE)
        self.limit = int(match.group(1))
        self.scope = [int(v) - 1 for v in match.group(2).split()]

    def test_feasibility(self, instance):
        return 0 < self.limit <= instance.k \
               and all([0 <= s < instance.k for s in self.scope]) \
               and len(self.scope) > 0

    def test_satisfiability(self, solution):
        U = set([solution.assignment[s] for s in self.scope])
        return len(U) <= self.limit


class Sual:
    def __init__(self):
        self.limit = -1
        self.scope = []
        self.user_group = []

    def read(self, line):
        match = re.match(r'sual scope\s+([\d\s]+)\s+limit\s+(\d+)\s+users\s+([\d\s]+)', line, re.IGNORECASE)
        self.scope = [int(v) - 1 for v in match.group(1).split()]
        self.limit = int(match.group(2))
        self.user_group = [int(v) - 1 for v in match.group(3).split()]

    def test_feasibility(self, instance):
        return 0 < self.limit <= instance.k \
               and all([0 <= s < instance.k for s in self.scope]) \
               and all([0 <= u < instance.n for u in self.user_group]) \
               and len(self.scope) > 0 \
               and len(self.user_group) > 0

    def test_satisfiability(self, solution):
        U = set([solution.assignment[s] for s in self.scope])
        return len(U) >= self.limit or U - set(self.user_group) == {}


class WangLi:
    def __init__(self):
        self.T = []
        self.user_groups = []

    def read(self, line):
        match = re.match(r'wang-li scope\s+([\d\s]+)\s+user groups\s+([\d\s\(\)]+)', line, re.IGNORECASE)
        self.T = [int(v) - 1 for v in match.group(1).split()]
        matches = re.findall(r'\(([\d\s]+)\)', match.group(2))
        self.user_groups = [[int(v) - 1 for v in m.split()] for m in matches]

    def test_feasibility(self, instance):
        return all([0 <= s < instance.k for s in self.T]) \
               and all([0 <= u < instance.n for team in self.user_groups for u in team]) \
               and len(self.user_groups) > 0 \
               and all([len(team) > 0 for team in self.user_groups])

    def test_satisfiability(self, solution):
        U = set([solution.assignment[s] for s in self.T])
        for group in self.user_groups:
            if U - set(group) == {}:
                return True

        return False


class AssignmentDependent:
    def __init__(self):
        self.s1 = -1
        self.s2 = -1
        self.U1 = []
        self.U2 = []

    def read(self, line):
        match = re.match(r'assignment-dependent scope\s+(\d+)\s+(\d+)\s+users\s+([\d\s]+)\s+and\s+([\d\s]+)', line, re.IGNORECASE)
        self.s1 = int(match.group(1)) - 1
        self.s2 = int(match.group(2)) - 1
        self.U1 = [int(v) - 1 for v in match.group(3).split()]
        self.U2 = [int(v) - 1 for v in match.group(4).split()]

    def test_feasibility(self, instance):
        return 0 <= self.s1 < instance.k \
               and 0 <= self.s2 < instance.k \
               and all([0 <= u < instance.n for u in self.U1]) \
               and all([0 <= u < instance.n for u in self.U2]) \
               and len(self.U1) > 0 \
               and len(self.U2) > 0

    def test_satisfiability(self, solution):
        u1 = solution.assignment[self.s1]
        u2 = solution.assignment[self.s2]
        if not (u1 not in self.U1 or u2 in self.U2):
            return False

        return u1 not in self.U1 or u2 in self.U2


class Solution:
    def __init__(self, assignment, time):
        self.assignment = assignment
        self.time = time

    def save(self, filename):
        f = open(filename, 'w')

        if not self.assignment:
            f.write('unsat\n')
        else:
            f.write('sat\n')

        f.write(f'{self.time}\n')
        if self.assignment:
            for s in range(len(self.assignment)):
                f.write(f'Step {s+1} -> User {self.assignment[s]+1}\n')

        f.close()

    def test_satisfiability(self, instance):
        for s in range(instance.k):
            if not instance.authorisations[self.assignment[s]].authorisation_list[s]:
                return False

        for c in instance.constraints:
            if not c.test_satisfiability(self):
                return False

        return True