import plotly as plt
import sys


def create_scatter_plot(instance_name: str, *story_paths: str):
    """
    :param instance: Instance Name eg. ta01
    :param file_name: solution type

    e.g. ./scatter.py ta01 hillclimber_1swap_restarts
    """
    fig = plt.graph_objs.Figure()
    fig.update_layout(title=f"JSSP - Instance: {instance_name.capitalize()}", legend_title="Algorithms:")

    for story in map(str.lower, story_paths):
        with open(f"../solutions/{instance_name}_{story}_history.txt", "r") as file:
            history = tuple(map(int, next(file).split()))
            fig.add_scatter(y=history, name=story.capitalize(), mode='markers')

    fig.show()


create_scatter_plot(*sys.argv[1:])
