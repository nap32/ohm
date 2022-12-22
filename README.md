# About
ΩΩΩΩΩΩΩΩΩΩ

### Design Goals
Ohm's goal is to streamline information gathering, storage, and interaction with a live system.
Ordered List of Priorities:
        - Correctness
        - Reliability
        - Extensibility
        - Efficiency
        - Usability
In practice, this means that sometimes using it requires digging in to the underlying system and requires upfront investment.
For example, GUIs are not a priority - as even though they are more usable, the author finds TUIs to be more efficient.

### Traffic v. Record
Two structs may appear similar and be confusing, but serve different purposes - traffic and records.
Traffic structs are the anecdote, Record structs are the unique endpoints.
Traffic should be immutable on creation - it is supposed to be a parsed representation of the actual traffic flows.
Records are mutable -
        - Records will include the same fields as Traffic, but the values won't be 1-to-1 with reproducible traffic.
            - Instead, if there is a recognized type variable, the recognized type literal be replaced with a symbol representing the type.
        - Records will also include a list of example traffics to demonstrate successful use of an endpoint.
        - Records will be the crux of the whole system - they can be extended to support markdown notes, tags and labels, etc.
