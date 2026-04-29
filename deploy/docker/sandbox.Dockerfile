# syntax=docker/dockerfile:1.7
# The image run_sandbox runs commands in.
FROM mirror.gcr.io/library/debian:bookworm-slim

# python3 with HTML, table, HTTP and document libraries, OCR, and common shell tools.
# PaddleOCR (Baidu, open source) reads scans; tesseract is the fallback.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        python3 \
        python3-bs4 \
        python3-lxml \
        python3-requests \
        python3-pandas \
        python3-dateutil \
        python3-yaml \
        python3-tabulate \
        python3-html2text \
        python3-pypdf \
        python3-docx \
        python3-openpyxl \
        python3-xlrd \
        python3-numpy \
        python3-pil \
        python3-chardet \
        python3-markdown \
        python3-venv \
        libgl1 \
        libglib2.0-0 \
        libgomp1 \
        tesseract-ocr \
        tesseract-ocr-eng \
        tesseract-ocr-chi-sim \
        tesseract-ocr-chi-tra \
        ca-certificates \
        curl \
        wget \
        jq \
        gawk \
        bc \
        coreutils \
        findutils \
        grep \
        sed \
        ripgrep \
        sqlite3 \
        poppler-utils \
        file \
        unzip \
        zip \
        less \
        tree \
    && rm -rf /var/lib/apt/lists/*

# PaddleOCR in its own environment, with the English and Chinese models fetched now, since the
# sandbox has no network at run time. PaddlePaddle publishes CPU wheels for amd64 only; other
# architectures build without it and read scans with tesseract.
ARG TARGETARCH
ARG PADDLEOCR=true
COPY deploy/sandbox/paddle_ocr.py /usr/local/bin/paddle-ocr
RUN if [ "$PADDLEOCR" = "true" ] && [ "$TARGETARCH" = "amd64" ]; then \
        python3 -m venv /opt/paddleocr/venv \
        && /opt/paddleocr/venv/bin/pip install --no-cache-dir \
            "numpy<2" paddlepaddle==2.6.2 paddleocr==2.8.1 \
        && HOME=/opt/paddleocr /opt/paddleocr/venv/bin/python -c \
            "from paddleocr import PaddleOCR; [PaddleOCR(use_angle_cls=True, lang=l, show_log=False) for l in ('en', 'ch')]" \
        && chmod -R a+rX /opt/paddleocr \
        && chmod 0755 /usr/local/bin/paddle-ocr; \
    else \
        rm -f /usr/local/bin/paddle-ocr; \
    fi

# Commands run as nobody against a read-only root, with the workspace mounted at /tmp.
USER 65534:65534
WORKDIR /tmp
ENV HOME=/tmp
ENTRYPOINT []
CMD ["sh"]
