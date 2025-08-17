import pytest
import nornir_core


def test_sum_as_string():
    assert nornir_core.sum_as_string(1, 1) == "2"
