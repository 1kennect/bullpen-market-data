#!/opt/paddleocr/venv/bin/python
"""Prints the text PaddleOCR reads in each image, one line per detected line of text."""

import argparse
import logging
import os
import sys

# Models are fetched at image build under this home; the sandbox has no network to fetch them.
os.environ["HOME"] = os.environ.get("PADDLEOCR_HOME", "/opt/paddleocr")


def main() -> int:
    parser = argparse.ArgumentParser(prog="paddle-ocr")
    parser.add_argument("--lang", default="en", help="PaddleOCR language code, such as en or ch")
    parser.add_argument("images", nargs="+")
    args = parser.parse_args()
    logging.disable(logging.WARNING)
    from paddleocr import PaddleOCR

    ocr = PaddleOCR(use_angle_cls=True, lang=args.lang, show_log=False)
    found = False
    for path in args.images:
        for page in ocr.ocr(path, cls=True) or []:
            for line in page or []:
                print(line[1][0])
                found = True
    return 0 if found else 1


if __name__ == "__main__":
    sys.exit(main())
