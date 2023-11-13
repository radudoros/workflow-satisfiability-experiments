import argparse
import json
import os
import subprocess
import time

from collections import defaultdict

import matplotlib.pyplot as plt


import sys
sys.path.append('./solvers')
from solver_ortools_pbpb import solve_pbpb
from datatypes import Instance

allowlist = ['ADA', 'AM3', 'SoD', 'SUAL', 'WL']  # Add the types you want here

time_limit = 180

# Custom sorting function
def custom_sort_key(instance_name):
    parts = instance_name.split()
    instance_type = parts[0]
    k_value = int(parts[1])
    n_value = int(parts[2])
    return (instance_type, k_value, n_value)

# Custom sorting function for instances
def instance_sort_key(instance_name):
    return int(instance_name.split('instance')[1].split('.txt')[0])

# Initialize a dictionary to hold average times by instance type
avg_times_by_type = defaultdict(lambda: {'k=18': [], 'k=18_baseline': [], 'n=10k': [], 'n=10k_baseline': []})

def main():
    parser = argparse.ArgumentParser(description='Run benchmarks.')
    parser.add_argument('--rust-binary-path', type=str, help='Path to the Rust binary')
    parser.add_argument('--root-dir', type=str, help='Root directory containing workloads')

    args = parser.parse_args()
    rust_binary_path = args.rust_binary_path
    root_dir = args.root_dir

    if os.path.exists('avg_times.json'):
        global avg_times_by_type
        avg_times_by_type = read_from_json('avg_times.json')

    # Loop through instance types (ADA, AM3, etc.)
    for instance_type in sorted(os.listdir(root_dir), key=custom_sort_key):
        main_type, k_value, n_value = instance_type.split()
        k_value, n_value = int(k_value), int(n_value)

        if main_type not in allowlist:
            continue

        if n_value == k_value * 10:
            if any(d.get('k', None) == k_value for d in avg_times_by_type.get(main_type, {}).get('n=10k', [])):
                print(f"Skipping because k={k_value} is already processed for n=10k")
                continue
        else:
            if any(d.get('n', None) == n_value for d in avg_times_by_type.get(main_type, {}).get('k=18', [])):
                print(f"Skipping because n={n_value} is already processed for k=18")
                continue

        # Skip if this k_value is already processed and stored in avg_times_by_type
        # if k_value in [d.get('k', None) for d in avg_times_by_type.get(main_type, {}).get('n=10k', [])]:
        #     # print(f'Skipping {instance_type} as k_value {k_value} is already processed.')
        #     continue

        # Stop if k is above the threshold (Note: This also messes up the runs for k = 18)
        if k_value > 28:
            # print(f'Skipping {instance_type} as n_value {n_value} is above threshold.')
            continue

        instance_type_path = os.path.join(root_dir, instance_type)

        if not os.path.isdir(instance_type_path):
            continue

        print(f'Processing instance type: {instance_type}')

        # Get list of instances sorted by their index
        sorted_instances = sorted(
            os.listdir(instance_type_path),
            key=instance_sort_key
        )
        
        fail_count = 0
        total_time = 0
        successful_runs = 0

        max_instances = 100  # Substitute with your desired maximum number of instances
        instances_run = 0  # Counter to keep track of instances run

        total_baseline = 0

        # Loop through sorted instances
        for instance in sorted_instances:
            instance_path = os.path.join(instance_type_path, instance)

            solver_intance = Instance(instance_path)
            solution = solve_pbpb(solver_intance)
            baseline_time = solution.time
            if solution.assignment:
                baseline_solution_found = True
            else:
                baseline_solution_found = False
            
            # Stop if max number of instances have been run
            if instances_run >= max_instances:
                print(f'Stopping: Max number of instances ({max_instances}) have been run.')
                break
            
            # Measure time and execute Rust binary
            start_time = time.time()
            cmd = f"timeout {time_limit} {rust_binary_path} \"{instance_path}\""
            result = subprocess.run(cmd, shell=True, capture_output=True)
            elapsed_time = time.time() - start_time

            stdout_str = result.stdout.decode('utf-8')
            if "Found a solution" in stdout_str:
                found_solution = True
            else:
                found_solution = False

            if baseline_solution_found != found_solution:
                print(f'We found a bug for {instance_path}')
                return

            instances_run += 1  # Increment the counter

            # Check for timeout
            if result.returncode != 0:
                print(f'Timeout for instance: {instance}')
                if result.stderr:
                    print(f'Error: {result.stderr}')
                fail_count += 1
            else:
                total_time += elapsed_time
                total_baseline += baseline_time
                successful_runs += 1

            # Stop running larger workloads if more than half have timed out
            if fail_count * 2 > len(sorted_instances):
                print(f'Stopping: more than half of the instances exceeded the time limit for type {instance_type}')
                break

        if successful_runs > 0:
            average_time = total_time / successful_runs
            average_baseline_time = total_baseline / successful_runs
            print(f'Average time for {instance_type}: {average_time}(ours) and {average_baseline_time}(baseline) seconds')

            # Store the average time by instance type and condition
            if k_value == 18:
                avg_times_by_type[main_type]['k=18'].append({'n': n_value, 'avg_time': average_time})
                avg_times_by_type[main_type]['k=18_baseline'].append({'n': n_value, 'avg_time': average_baseline_time})
            if n_value == 10 * k_value:
                avg_times_by_type[main_type]['n=10k'].append({'k': k_value, 'avg_time': average_time})
                avg_times_by_type[main_type]['n=10k_baseline'].append({'k': k_value, 'avg_time': average_baseline_time})

        # incremental: save each time we run
        save_to_json(avg_times_by_type, 'avg_times.json')
    

# Function to plot line chart by condition
def plot_chart(condition):
    plt.figure()
    plt.title(f'Average Time for {condition}')
    plt.ylabel('Average Time (seconds)')

    # Set y-axis to be logarithmic
    plt.yscale('log')

    # Set y-axis limits
    plt.ylim(10**-2, 10**4)

    if condition == 'k=18':
        plt.xlabel('n value')
        plt.xscale('log')
    elif condition == 'n=10k':
        plt.xlabel('k value')
        plt.xlim(15, 60)


    for instance_type, conditions in avg_times_by_type.items():
        avg_times = conditions[condition]
        avg_baseline_times = conditions[f"{condition}_baseline"]
        x_values = sorted([d['k' if condition == 'n=10k' else 'n'] for d in avg_times])
        y_values = [d['avg_time'] for d in sorted(avg_times, key=lambda d: d['k' if condition == 'n=10k' else 'n'])]
        x_values_baseline = sorted([d['k' if condition == 'n=10k' else 'n'] for d in avg_baseline_times])
        y_values_baseline = [d['avg_time'] for d in sorted(avg_baseline_times, key=lambda d: d['k' if condition == 'n=10k' else 'n'])]

        plt.plot(x_values, y_values, label=instance_type)
        plt.plot(x_values_baseline, y_values_baseline, linestyle='--', label=f"{instance_type} Baseline")


    plt.legend()
    plt.savefig(f'Average_Time_{condition}.png')
    print(f'Figures saved for {condition}')

def save_to_json(data, filename):
    with open(filename, 'w') as f:
        json.dump(data, f)

def read_from_json(filename):
    with open(filename, 'r') as f:
        return json.load(f)

if __name__ == "__main__":
    main()

# Generate line chart for k = 18
plot_chart('k=18')

# Generate line chart for n = 10k
plot_chart('n=10k')