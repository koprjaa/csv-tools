#
# Project: csv-tools
# File:    __main__.py
#
# Description:
# Lets the package run as `python -m csv_tools`.
#
# Author:
# Jan Alexandr Kopřiva
# jan.alexandr.kopriva@gmail.com
#
# License: MIT
#

"""Allow `python -m csv_tools`."""

import sys

from csv_tools.cli import main

sys.exit(main())
