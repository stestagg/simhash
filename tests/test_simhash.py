import pytest
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
    hasher = simhash.SimHasher()
    sh1 = hasher.hash("The cat sat on the mat")
    sh2 = hasher.hash("The cat spat on the mat")

    assert sh1 != sh2
    assert sh1.difference(sh2) == 4

def test_hash_xxh3_2byte():
    hasher = simhash.SimHasher(hash_method=simhash.HashMethod.XXHash)
    sh1 = hasher.hash("The cat sat on the mat")
    sh2 = hasher.hash("The cat spat on the mat")

    assert sh1 != sh2
    assert sh1.difference(sh2) == 4

def test_hash_difference():
    sh1 = simhash.hash("The cat sat on the mat", method=simhash.HashMethod.SipHash)
    sh2 = simhash.hash("The cat spat on the mat", method=simhash.HashMethod.XXHash)

    assert sh1 != sh2

def test_odd_ones():
    assert simhash.hash('') == simhash.SimHash.from_int(0)
    assert simhash.hash('', method=simhash.HashMethod.XXHash) == simhash.SimHash.from_int(0)
    assert simhash.hash('a', n=1).value != 0
    assert simhash.hash('a', n=2).value == 0
    assert simhash.hash('a', method=simhash.HashMethod.XXHash, n=1).value != 0


def test_variance():
    variances = []
    vocab = string.ascii_letters + string.digits + string.punctuation
    for base_len in range(1, 65):
        base = ''.join(vocab[i % len(vocab)] for i in range(base_len))
        base_sip = simhash.hash(base)

        vars = []
        for c in vocab:
            test_str = base + c
            test_sip = simhash.hash(test_str)
            vars.append(base_sip.difference(test_sip))
        avg = sum(vars) / len(vars)
        variances.append(int(avg))

    first_avg = sum(variances[:8]) / 8
    last_avg = sum(variances[-8:]) / 8
    assert first_avg > last_avg * 2
    
# def test_bytes_input():
#     sh1 = simhash.hash(b"The cat sat on the mat")
#     sh2 = simhash.hash("The cat sat on the mat")
#     sh3 = simhash.hash(bytearray(b"The cat sat on the mat"))
#     sh4 = simhash.hash(b"Bob")

#     assert sh1 == sh2
#     assert sh1 == sh3
#     assert sh1 != sh4

def test_all_byte_vals():
    all_bytes = bytes(range(256)) * 2
    all_bytes = all_bytes.decode('latin1')
    sh1 = simhash.hash(all_bytes)
    sh2 = simhash.hash(all_bytes, method=simhash.HashMethod.XXHash)
    sh3 = simhash.hash(all_bytes, features=simhash.FeatureType.Bytes, n=2)

    assert sh1.value != 0
    assert sh2.value != 0
    assert sh3.value != 0

def test_features():
    # Test a string with emojis and accented characters
    test_str = "One 🐈‍⬛ sat on the 🪑, the other 🐈‍🟫 was 🏃🏽‍♀️"
    assert len(test_str) == 45
    features_bytes = list(simhash.features(test_str, simhash.FeatureType.Bytes))
    features_graphemes = list(simhash.features(test_str, simhash.FeatureType.Graphemes))

    assert [b.decode(errors='replace') for b in features_bytes] == [
        'O', 'n', 'e', ' ', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', ' ', 
        's', 'a', 't', ' ', 'o', 'n', ' ', 't', 'h', 'e', ' ', '�', '�', '�', '�', ',', 
        ' ', 't', 'h', 'e', ' ', 'o', 't', 'h', 'e', 'r', ' ', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', ' ', 'w', 'a', 's', 
        ' ', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�', '�']
    assert features_graphemes == [
        'O', 'n', 'e', ' ', '🐈\u200d⬛', ' ', 's', 'a', 't', ' ', 'o', 'n', ' ', 't', 'h', 'e', 
        ' ', '🪑', ',', ' ', 't', 'h', 'e', ' ', 'o', 't', 'h', 'e', 'r', ' ', '🐈\u200d🟫', ' ', 'w', 'a', 's', ' ', '🏃🏽\u200d♀️']
    
    reconstructed = (b''.join(features_bytes)).decode('utf-8')
    assert reconstructed == test_str
    reconstructed_graphemes = ''.join(features_graphemes)
    assert reconstructed_graphemes == test_str

def test_word_features():
    test_str = "One 🐈‍⬛ sat on the 🪑, the other 🐈‍🟫 was 🏃🏽‍♀️"
    features_words = simhash.features(test_str, simhash.FeatureType.Words)
    assert features_words == ['One', 'sat', 'on', 'the', 'the', 'other', 'was']


def test_grouping():
    texts = [
        "The cat sat on the mat",
        "The cat spat on the mat",
        "A dog barked all the way to the $MOON",
        "A doge barked all the way to the $MOON",
    ]
    groups = simhash.group_texts(texts, max_diff=6)
    assert len(groups) == 2
    assert sorted(groups, key=lambda g: g[0]) == [
        [
            "A dog barked all the way to the $MOON",
            "A doge barked all the way to the $MOON",
        ],
        [
            "The cat sat on the mat",
            "The cat spat on the mat",
        ],
    ]