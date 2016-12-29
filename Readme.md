# BACnet-rs

master : [![Build Status](https://travis-ci.org/platy/bacnet-rs.svg?branch=master)](https://travis-ci.org/platy/bacnet-rs)

Currently this consists only of an abstract-syntax-tree representation of BACnet values and parsers for those trees.

## Architecture

## Serialisation of messages

BACnet has a fairly complicated serialisation, so the first piece of architecture here is to (de)serialise between the byte replesentation and an abstract syntax tree (AST). The AST can then be marshalled back and forth into messages which are passed to the application. This separation means that structural logic is handled in the marshalling and serialisation logic is handled in the serialisation - giving a good separation. This will not be as efficient as doing it together but I think it will be worth it.

```
       >> Deserialise >> Unmarshal >>
Serial form          AST         Message
         << Serialise << Marshal <<
```

## TODO

- AST
- Parsers
- Serialisers
- Marshallers
- Unmarshallers
- Services ...
- Object model interface
- Network layer

## Milestones

### 0.1 - First release

- Rewrite readme
- Examples for several use profiles
- Put in crates.io repository
- Reviewed by severl relevent people
- Describe milestone 0.2 - Including a new profile fully supported - Annex L
- Comply to style guidelines
- Support local networking at least
- Support whois, iam, who-has, i-have, readprop, Cov, writeprop BIBBs - Annex K
- Describe support in terms of BIBBs - Annex K
- Support device, binaryx and analogx object types
- Support properties for the above object types
- Support all the Application PDU types in Clause 20
- Support includes all of B-General - Annex L
- Register vendor id to use in examples
- Consider ASN.1 serialisers instead of having our own implementation and at least document that ASN.1 is what is being used

