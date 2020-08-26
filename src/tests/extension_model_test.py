#!/usr/bin/env python

# Copyright (C) 2020 The Nitrocli Developers
# SPDX-License-Identifier: GPL-3.0-or-later

from argparse import (
  ArgumentParser,
)
from enum import (
  Enum,
)
from os import (
  environ,
)
from sys import (
  argv,
  exit,
)


class Action(Enum):
  """An action to perform."""
  NITROCLI = "nitrocli"
  MODEL = "model"
  VERBOSITY = "verbosity"

  @classmethod
  def all(cls):
    """Return the list of all the enum members' values."""
    return [x.value for x in cls.__members__.values()]


def main(args):
  """The extension's main function."""
  parser = ArgumentParser()
  parser.add_argument(choices=Action.all(), dest="what")
  parser.add_argument("--nitrocli", action="store", default=None)
  parser.add_argument("--model", action="store", default=None)
  # We deliberately store the argument to this option as a string
  # because we can differentiate between None and a valid value, in
  # order to verify that it really is supplied.
  parser.add_argument("--verbosity", action="store", default=None)

  namespace = parser.parse_args(args[1:])
  if namespace.what == Action.NITROCLI.value:
    print(environ["NITROCLI_BINARY"])
  elif namespace.what == Action.MODEL.value:
    print(environ["NITROCLI_MODEL"])
  elif namespace.what == Action.VERBOSITY.value:
    print(environ["NITROCLI_VERBOSITY"])
  else:
    return 1

  return 0


if __name__ == "__main__":
  exit(main(argv))
