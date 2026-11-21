# udsdiag

An offline model of the UDS (ISO 14229-1:2013) and GM GMLAN diagnostic services, split out of
the `volt` GM head-unit and OBD-app assessment so the protocol model is reusable apart from the
research write-ups.

## This package

- The diagnostic service catalog (`services.py`) and the negative-response-code catalog
  (`nrc.py`), from ISO 14229-1:2013.
- The request/response codec (`message.py`): build a request, decode a positive or negative
  response, recognise response-pending. Application PDU only.
- The reference catalogs recovered in the assessment: the DID set (`did.py`), the DTC model and
  a representative subset (`dtc.py`), the RoutineControl and IOControl catalogs (`routine.py`,
  `iocontrol.py`), the SecurityAccess exchange model and mechanism descriptors (`security.py`),
  the GM Mode $06 document catalog (`mode06.py`), and GMLAN addressing (`gmlan.py`).
- A read-only CLI over all of it (`cli.py`).

## Not this package

- **No transport.** The ISO-TP (ISO 15765-2) codec that segments and reassembles a PDU is the
  sibling `isotpkit`. `message.py` works on the reassembled PDU.
- **No vehicle I/O and no key material.** No CAN interface, no vehicle write, no SecurityAccess
  seed→key algorithm or key.
- **Not the full DTC database.** The complete 841-code app table joined to GM 17OBDG01 is the
  sibling `gmdtcdb`; this package carries a representative subset and the counts.
- **Not the Mode $06 parser.** Parsing GM's Mode $06 PDFs into records is the sibling
  `mode06kit`; this package catalogs the documents only.

## Siblings

`isotpkit` (transport under this), `mode06kit` (Mode $06 parser), `gmdtcdb` (full DTC
database), `voltkit` / `spskit` / `techlinekit` (the firmware-read, reflash and TDS-client
sides). None imports another. All trace to the same read-only `volt` assessment.
