"""Atlas Test Framework for C2PA ML Provenance Testing"""

__version__ = "0.1.0"

from .framework import AtlasTestFramework
from .runner import main

__all__ = ["AtlasTestFramework", "main"]