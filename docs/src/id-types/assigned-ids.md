# Assigned IDs

idt supports validation and inspection of assigned identifiers. Unlike generated IDs (UUID, ULID, etc.), these are **not generated** by idt — they are issued by external registries, publishers, and standards bodies.

## EAN-13

International Article Number — the standard barcode for retail products worldwide.

| Property | Value |
|----------|-------|
| Length | 13 digits |
| Check Digit | Mod 10 (alternating weights 1, 3) |
| Sortable | No |
| Timestamp | No |

### Format

```
4006381333931
|-----------|X
 12 digits   check digit
```

### Validation

```bash
idt validate -t ean13 4006381333931
# Output: 4006381333931: valid (ean13)

idt inspect 4006381333931
```

### Specification

[GS1 - EAN/UPC](https://www.gs1.org/standards/barcodes/ean-upc)

---

## ISBN-13

International Standard Book Number (current 13-digit format, since 2007).

| Property | Value |
|----------|-------|
| Length | 13 digits |
| Prefix | 978 or 979 |
| Check Digit | Mod 10 (alternating weights 1, 3) |
| Sortable | No |
| Timestamp | No |

### Format

```
978-0-306-40615-7
|---|            |
prefix     check digit
```

ISBN-13 is a subset of EAN-13 with the 978 or 979 prefix.

### Conversion

ISBN-13 with prefix 978 can be converted to ISBN-10:

```bash
idt inspect 9780306406157
# Components include: isbn10: "0306406152"
```

### Validation

```bash
idt validate -t isbn13 978-0-306-40615-7
idt validate -t isbn13 9780306406157
```

### Specification

[ISBN International](https://www.isbn-international.org/)

---

## ISBN-10

International Standard Book Number (legacy 10-character format, pre-2007).

| Property | Value |
|----------|-------|
| Length | 10 characters (9 digits + check) |
| Check Digit | Mod 11 (weighted), can be 0-9 or X |
| Sortable | No |
| Timestamp | No |

### Format

```
0-306-40615-2
|---------|X
9 digits   check digit (0-9 or X)
```

The check digit X represents the value 10 in the Mod 11 algorithm.

### Conversion

Every ISBN-10 can be converted to ISBN-13 by prepending 978 and recalculating the check digit:

```bash
idt inspect 0306406152
# Components include: isbn13: "9780306406157"
```

### Validation

```bash
idt validate -t isbn10 0306406152
idt validate -t isbn10 080442957X     # X as check digit
idt validate -t isbn10 0-306-40615-2  # hyphens are stripped
```

### Specification

[ISBN International](https://www.isbn-international.org/)

---

## ISIN

International Securities Identification Number — uniquely identifies securities (stocks, bonds, derivatives).

| Property | Value |
|----------|-------|
| Length | 12 characters |
| Structure | 2-letter country code + 9 alphanumeric + 1 check digit |
| Check Digit | Luhn algorithm (letters converted: A=10, B=11, ..., Z=35) |
| Sortable | No |
| Timestamp | No |

### Format

```
US0378331005
|--|-------|X
CC  NSIN    check digit
```

- **CC**: ISO 3166-1 alpha-2 country code
- **NSIN**: National Securities Identifying Number (9 alphanumeric characters)
- **Check digit**: Luhn algorithm applied after converting letters to numbers

### Validation

```bash
idt validate -t isin US0378331005     # Apple Inc.
idt validate -t isin GB0002634946     # BAE Systems
idt validate -t isin AU0000XVGZA3     # Australian security
```

### Inspection

```bash
idt inspect US0378331005
# Components: country_code: "US", nsin: "037833100", check_digit: "5"
```

### Specification

[ISO 6166](https://www.iso.org/standard/78502.html)

---

---

## EAN-8

8-digit barcode for small items where EAN-13 is too large.

| Property | Value |
|----------|-------|
| Length | 8 digits |
| Check Digit | Mod 10 (alternating weights 1, 3) |
| Sortable | No |
| Timestamp | No |

### Format

```
96385074
|------|X
 7 digits check digit
```

### Validation

```bash
idt validate -t ean8 96385074
# Output: 96385074: valid (ean8)
```

### Specification

[GS1 - EAN/UPC](https://www.gs1.org/standards/barcodes/ean-upc)

---

## UPC-A

Universal Product Code — the standard 12-digit barcode used in North America.

| Property | Value |
|----------|-------|
| Length | 12 digits |
| Check Digit | Mod 10 (alternating weights 1, 3) |
| Sortable | No |
| Timestamp | No |

### Format

```
036000291452
|----------|X
 11 digits  check digit
```

### Conversion

UPC-A can be converted to EAN-13 by prepending a leading zero:

```bash
idt inspect 036000291452
# Components include: ean13: "0036000291452"
```

### Validation

```bash
idt validate -t upca 036000291452
```

### Specification

[GS1 - EAN/UPC](https://www.gs1.org/standards/barcodes/ean-upc)

---

## ISSN

International Standard Serial Number — identifies serial publications (journals, magazines, newspapers).

| Property | Value |
|----------|-------|
| Length | 8 characters |
| Format | XXXX-XXXX |
| Check Digit | Mod 11 (weighted), can be 0-9 or X |
| Sortable | No |
| Timestamp | No |

### Format

```
0378-5955
|------|X
7 digits check digit (0-9 or X)
```

### Validation

```bash
idt validate -t issn 0378-5955
idt validate -t issn 03785955     # without hyphen
```

### Specification

[ISSN International Centre](https://www.issn.org/)

---

## ISMN

International Standard Music Number — identifies printed music publications.

| Property | Value |
|----------|-------|
| Length | 13 digits |
| Prefix | 979-0 |
| Check Digit | Mod 10 (alternating weights 1, 3) |
| Sortable | No |
| Timestamp | No |

### Format

```
979-0-060-11561-5
|---|             |
prefix      check digit
```

ISMN is a subset of EAN-13 with the 979-0 prefix.

### Validation

```bash
idt validate -t ismn 979-0-060-11561-5
idt validate -t ismn 9790060115615
```

### Specification

[ISMN International Agency](https://ismn-international.org/)

---

## ISNI

International Standard Name Identifier — uniquely identifies contributors to creative works.

| Property | Value |
|----------|-------|
| Length | 16 characters |
| Format | XXXX XXXX XXXX XXXX |
| Check Digit | ISO 7064 MOD 11-2 (last char can be 0-9 or X) |
| Sortable | No |
| Timestamp | No |

### Format

```
0000 0001 2103 2683
|--- ---- ---- ---|X
 15 digits         check digit
```

### Validation

```bash
idt validate -t isni "0000 0001 2103 2683"
idt validate -t isni 0000000121032683
```

### Specification

[ISNI International Authority](https://isni.org/)

---

## GTIN-14

Global Trade Item Number — identifies trade items at various packaging levels.

| Property | Value |
|----------|-------|
| Length | 14 digits |
| Check Digit | Mod 10 (alternating weights 1, 3) |
| Sortable | No |
| Timestamp | No |

### Format

```
10614141000415
X|-----------|X
PI            check digit
```

- **PI**: Packaging indicator (first digit)

### Validation

```bash
idt validate -t gtin14 10614141000415
```

### Specification

[GS1 - GTIN](https://www.gs1.org/standards/id-keys/gtin)

---

## ASIN

Amazon Standard Identification Number — Amazon's proprietary product identifier.

| Property | Value |
|----------|-------|
| Length | 10 characters |
| Characters | Alphanumeric (uppercase) |
| Check Digit | None (format validation only) |
| Sortable | No |
| Timestamp | No |

### Format

```
B08N5WRWNW
```

ASINs starting with `B` are Amazon-assigned. ASINs starting with a digit are typically ISBN-10 based.

### Validation

```bash
idt validate -t asin B08N5WRWNW
```

---

## Comparison

| Type | Length | Characters | Check Algorithm | Use Case |
|------|--------|------------|-----------------|----------|
| EAN-13 | 13 | Digits | Mod 10 (1,3) | Retail barcodes |
| EAN-8 | 8 | Digits | Mod 10 (1,3) | Small item barcodes |
| UPC-A | 12 | Digits | Mod 10 (1,3) | North American products |
| ISBN-13 | 13 | Digits | Mod 10 (1,3) | Books |
| ISBN-10 | 10 | Digits + X | Mod 11 | Books (legacy) |
| ISSN | 8 | Digits + X | Mod 11 | Serial publications |
| ISMN | 13 | Digits | Mod 10 (1,3) | Printed music |
| ISNI | 16 | Digits + X | ISO 7064 MOD 11-2 | Name identifiers |
| ISIN | 12 | Alphanumeric | Luhn | Securities |
| GTIN-14 | 14 | Digits | Mod 10 (1,3) | Trade items |
| ASIN | 10 | Alphanumeric | None | Amazon products |

## Support Status

| Type | Generate | Inspect | Convert | Validate |
|------|----------|---------|---------|----------|
| EAN-13 | No | Yes | Yes | Yes |
| EAN-8 | No | Yes | Yes | Yes |
| UPC-A | No | Yes | Yes | Yes |
| ISBN-13 | No | Yes | Yes | Yes |
| ISBN-10 | No | Yes | Yes | Yes |
| ISSN | No | Yes | Yes | Yes |
| ISMN | No | Yes | Yes | Yes |
| ISNI | No | Yes | Yes | Yes |
| ISIN | No | Yes | Yes | Yes |
| GTIN-14 | No | Yes | Yes | Yes |
| ASIN | No | Yes | Yes | Yes |
