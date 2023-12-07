import json
import os
import matplotlib.pyplot as plt
from collections import defaultdict

def consolidate_results(task_nr, output_filename):
    consolidated_data = defaultdict(lambda: defaultdict(list))

    for task_id in range(task_nr):
        filename = f'avg_times_{task_id}.json'
        if os.path.exists(filename):
            with open(filename, 'r') as f:
                data = json.load(f)
                for instance_type, conditions in data.items():
                    for condition, times_list in conditions.items():
                        for time_entry in times_list:
                            n_value = time_entry['n']
                            avg_time = time_entry['avg_time']

                            # Find if this 'n' value already exists in consolidated data
                            existing_entry = next((entry for entry in consolidated_data[instance_type][condition] if entry['n'] == n_value), None)
                            if existing_entry:
                                existing_entry['avg_time'] += avg_time
                            else:
                                consolidated_data[instance_type][condition].append({'n': n_value, 'avg_time': avg_time})

    # Average the 'avg_time' for each entry
    for instance_type, conditions in consolidated_data.items():
        for condition, times_list in conditions.items():
            for time_entry in times_list:
                time_entry['avg_time'] /= task_nr

    # Write consolidated results
    with open(output_filename, 'w') as f:
        json.dump(consolidated_data, f)

    return consolidated_data


# Function to plot line chart by condition
def plot_chart(consolidated_data, condition):
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
    elif condition == 'n=k':
        plt.xlabel('k value (where n=k)')


    for instance_type, conditions in consolidated_data.items():
        avg_times = conditions[condition]
        avg_baseline_times = conditions[f"{condition}_baseline"]

        sort_key = 'n'  # Default case, adjust if necessary

        x_values = sorted([d[sort_key] for d in avg_times])
        y_values = [d['avg_time'] for d in sorted(avg_times, key=lambda d: d[sort_key])]
        x_values_baseline = sorted([d[sort_key] for d in avg_baseline_times])
        y_values_baseline = [d['avg_time'] for d in sorted(avg_baseline_times, key=lambda d: d[sort_key])]

        plt.plot(x_values, y_values, label=instance_type)
        plt.plot(x_values_baseline, y_values_baseline, linestyle='--', label=f"{instance_type} Baseline")


    plt.legend()
    plt.savefig(f'Average_Time_{condition}.png')
    print(f'Figures saved for {condition}')

def main():
    task_nr = 2  # Number of tasks to consolidate
    consolidated_data = consolidate_results(task_nr, output_filename='consolidated_results.json')

    # Plot charts for each condition
    for condition in ['n=k', 'k=18', 'n=10k']:
        plot_chart(consolidated_data, condition)

if __name__ == "__main__":
    main()