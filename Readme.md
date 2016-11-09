# Rs-BACnet

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

