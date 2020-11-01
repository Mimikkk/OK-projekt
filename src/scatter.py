import plotly as plt
import sys


def create_scatter_plot(instance_name: str, *story_paths: str):
    fig = plt.graph_objs.Figure()
    fig.update_layout(title=f"JSSP - Instance: {instance_name.capitalize()}", legend_title="Algorithms:")

    for story in story_paths:
        with open(f"../solutions/{instance_name}_{story}.txt", "r") as file:
            name = next(file)
            history = tuple(map(int, next(file).split()))
            fig.add_scatter(y=history, name=name, mode='markers')

    fig.show()


create_scatter_plot(*sys.argv[1:])
