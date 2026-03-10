# test_mypy.py
from sieve import sift, SiftLevel, SiftResult

def run_test() -> None:
    # mypy should know this takes a string and optional SiftLevel
    res: SiftResult = sift("Hello, please summarize this.", level=SiftLevel.High)
    
    # mypy should know these properties exist and their types
    print(f"Original (str): {res.original}")
    print(f"Sifted (str): {res.sifted}")
    print(f"Removed (int): {res.tokens_removed}")
    print(f"Ratio (float): {res.compression_ratio}")

if __name__ == "__main__":
    run_test()
