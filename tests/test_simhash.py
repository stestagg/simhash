import simhash
import string


def test_simhash_from_int():
    sh = simhash.SimHash.from_int(123)
    assert sh.value == 123
    assert str(sh) == "0x000000000000007b"
    assert repr(sh) == "<SimHash 0x000000000000007b>"
    assert hash(sh) == 123
    assert int(sh) == 123


def test_simhash_equality():
    sh1 = simhash.SimHash.from_int(123)
    sh2 = simhash.SimHash.from_int(123)
    sh3 = simhash.SimHash.from_int(456)
    assert sh1 == sh2
    assert sh1 != sh3


def test_difference():
    sh1 = simhash.SimHash.from_int(0b101010)
    sh2 = simhash.SimHash.from_int(0b101110)
    sh3 = simhash.SimHash.from_int(0b100010)
    assert sh1.difference(sh2) == 1
    assert sh2.difference(sh1) == 1
    assert sh1.difference(sh3) == 1
    assert sh2.difference(sh3) == 2

    empty = simhash.SimHash.from_int(0)
    full = simhash.SimHash.from_int(0xFFFFFFFFFFFFFFFF)
    assert empty.difference(full) == 64


def test_hash_sip_2byte():
    sh1 = simhash.hash_sip_2byte("The cat sat on the mat")
    sh2 = simhash.hash_sip_2byte("The cat spat on the mat")

    assert sh1 != sh2
    assert sh1.difference(sh2) == 4

def test_hash_xxh3_2byte():
    sh1 = simhash.hash_xxh3_2byte("The cat sat on the mat")
    sh2 = simhash.hash_xxh3_2byte("The cat spat on the mat")

    assert sh1 != sh2
    assert sh1.difference(sh2) == 4

def test_hash_difference():
    sh1 = simhash.hash_sip_2byte("The cat sat on the mat")
    sh2 = simhash.hash_xxh3_2byte("The cat spat on the mat")

    assert sh1 != sh2

def test_odd_ones():
    assert simhash.hash_sip_2byte('') == simhash.SimHash.from_int(0)
    assert simhash.hash_xxh3_2byte('') == simhash.SimHash.from_int(0)
    assert simhash.hash_sip_2byte('a').value != 0
    assert simhash.hash_xxh3_2byte('a').value != 0


def test_variance():
    variances = []
    vocab = string.ascii_letters + string.digits + string.punctuation
    for base_len in range(1, 65):
        base = ''.join(vocab[i % len(vocab)] for i in range(base_len))
        base_sip = simhash.hash_sip_2byte(base)

        vars = []
        for c in vocab:
            test_str = base + c
            test_sip = simhash.hash_sip_2byte(test_str)
            vars.append(base_sip.difference(test_sip))
        avg = sum(vars) / len(vars)
        variances.append(int(avg))

    first_avg = sum(variances[:8]) / 8
    last_avg = sum(variances[-8:]) / 8
    assert first_avg > last_avg * 2
    
