import simhash


def test_simdict_add():
    sd = simhash.SimDict()
    sd['The cat sat on the mat'] = 1
    assert len(sd) == 1
    assert "The cat sat on the mat" in sd
    assert sd.get("The cat sat on the mat") == 1

def test_simdict_same_hash():
    sd = simhash.SimDict(max_diff=6)
    base = "The cat sat on the mat"
    sd[base] = 1
    assert len(sd) == 1
    assert base + '!' in sd
    assert 'bob the builder' not in sd
    sd[base + '!'] = 2
    assert sd[base + '!'] == 1
    assert sd[base] == 1