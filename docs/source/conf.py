import os
import sys

sys.path.insert(0, os.path.abspath("../../python"))

project = "arclib"
copyright = ""
author = ""

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
]

graphs_path = ["_graphs"]
exclude_patterns = []

html_theme = "sphinx_rtd_theme"
