---------------------------- MODULE SharedAccess ----------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Formal specification of WORTH Shared Access (Daymark Wave 4, P4.2 / G11):
  Single-owner Directory Home with durable WORTH-governed scope capability
  grants, fail-closed remote read/watch delegation, constant-time authenticated
  sessions, immediate revocation, and Directory Home close/reopen durability.

  Safety invariants & action properties:
    - NeverLeakUngrantedScope (standalone invariant with per-(p,s) delivery witness)
    - NeverServeAfterRevoke (state invariant + universal step property)
    - NeverServeUnauthenticatedProp (universal step property across all (p,s))
    - GrantsDurableAcrossReopen (state invariant)
    - NoConcurrentWriters (state invariant)
    - NoMutationWhileClosedProp (step property)

  Liveness properties:
    - CollaboratorEventuallyObservesProject1Write
    - WatchEventuallyDeliversProject1Write
*)

CONSTANTS
    Scopes,          \* e.g. {"Project1", "Project2"}
    Principals,      \* e.g. {"Alice", "Bob"}
    MaxVersion,      \* e.g. 1 (exhaustive TLC check) or 64 (trace validation)
    MaxIncarnation   \* e.g. 2 (exhaustive TLC check) or 32 (trace validation)

VARIABLES
    homeOpen,           \* BOOLEAN: whether the single-owner Directory Home is open
    incarnation,        \* 1..MaxIncarnation: monotonic Directory Home open incarnation
    ownerWriter,        \* {"Owner", "None"}: single-owner writer lock
    scopeValue,         \* [Scopes -> 0..MaxVersion]: durable authoritative value per scope
    durableGrants,      \* [Principals \X Scopes -> GrantStates]: durable WORTH grants
    authStatus,         \* [Principals -> {"AuthValid", "AuthInvalid"}]: token validity
    watches,            \* [Principals \X Scopes -> {"Closed", "Open"}]: server-owned watches
    watchSeen,          \* [Principals \X Scopes -> 0..MaxVersion]: last version delivered on open watch
    lastDelivered,      \* [Principals \X Scopes -> 0..MaxVersion]: high-water version observed per (p,s) for liveness
    lastDelivery,       \* Bounded step-level delivery event witness (NoDelivery on non-delivery steps)
    closedGrants,       \* durableGrants snapshot while closed (EmptyGrants while open)
    closedValues,       \* scopeValue snapshot while closed (ZeroValues while open)
    reopenPreserved     \* BOOLEAN: TRUE iff latest ReopenHome preserved durableGrants and scopeValue

vars == <<
    homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
    authStatus, watches, watchSeen, lastDelivered, lastDelivery,
    closedGrants, closedValues, reopenPreserved
>>

GrantStates == {"None", "ActiveReadOnly", "ActiveReadAndWatch", "Revoked"}
AccessLevels == {"ReadOnly", "ReadAndWatch"}
AuthStates == {"AuthValid", "AuthInvalid"}
WatchStates == {"Closed", "Open"}
DeliveryKinds == {"Read", "Watch"}

EmptyGrants == [ps \in Principals \X Scopes |-> "None"]
ZeroValues == [s \in Scopes |-> 0]
ClosedWatches == [ps \in Principals \X Scopes |-> "Closed"]
ZeroPairVersions == [ps \in Principals \X Scopes |-> 0]
NoDelivery == [active |-> FALSE]

DeliveryEvent(p, s, kind, v) ==
    [active |-> TRUE, principal |-> p, scope |-> s, kind |-> kind, version |-> v]

DeliveryEvents ==
    {NoDelivery}
    \cup [
        active : {TRUE},
        principal : Principals,
        scope : Scopes,
        kind : DeliveryKinds,
        version : 0..MaxVersion
    ]

OutcomeKinds == {
    "Ok",
    "Delivered",
    "Pending",
    "AuthInvalidSignature",
    "AuthzCapabilityDenied",
    "AuthzWatchCapabilityRevoked",
    "WatchNotFound",
    "HomeClosed"
}

RowRecords == [scope : Scopes, version : 0..MaxVersion]
SingletonRow(s, v) == <<[scope |-> s, version |-> v]>>
EmptyRows == <<>>

HasReadGrant(g) == g \in {"ActiveReadOnly", "ActiveReadAndWatch"}
HasWatchGrant(g) == g = "ActiveReadAndWatch"

GrantStateForAccess(access) ==
    IF access = "ReadOnly" THEN "ActiveReadOnly" ELSE "ActiveReadAndWatch"

RecordDelivery(p, s, v) ==
    [lastDelivered EXCEPT ![<<p, s>>] = IF v > @ THEN v ELSE @]

TypeOK ==
    /\ homeOpen \in BOOLEAN
    /\ incarnation \in 1..MaxIncarnation
    /\ ownerWriter \in {"Owner", "None"}
    /\ scopeValue \in [Scopes -> 0..MaxVersion]
    /\ durableGrants \in [Principals \X Scopes -> GrantStates]
    /\ authStatus \in [Principals -> AuthStates]
    /\ watches \in [Principals \X Scopes -> WatchStates]
    /\ watchSeen \in [Principals \X Scopes -> 0..MaxVersion]
    /\ lastDelivered \in [Principals \X Scopes -> 0..MaxVersion]
    /\ lastDelivery \in DeliveryEvents
    /\ closedGrants \in [Principals \X Scopes -> GrantStates]
    /\ closedValues \in [Scopes -> 0..MaxVersion]
    /\ reopenPreserved \in BOOLEAN

Init ==
    /\ homeOpen = TRUE
    /\ incarnation = 1
    /\ ownerWriter = "Owner"
    /\ scopeValue = ZeroValues
    /\ durableGrants = EmptyGrants
    /\ authStatus = [p \in Principals |-> "AuthValid"]
    /\ watches = ClosedWatches
    /\ watchSeen = ZeroPairVersions
    /\ lastDelivered = ZeroPairVersions
    /\ lastDelivery = NoDelivery
    /\ closedGrants = EmptyGrants
    /\ closedValues = ZeroValues
    /\ reopenPreserved = TRUE

\* --- Owner-only mutations (single-owner Directory Home invariant) ---

OwnerWrite(s) ==
    /\ homeOpen
    /\ ownerWriter = "Owner"
    /\ scopeValue[s] < MaxVersion
    /\ scopeValue' = [scopeValue EXCEPT ![s] = @ + 1]
    /\ lastDelivery' = NoDelivery
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, durableGrants,
           authStatus, watches, watchSeen, lastDelivered,
           closedGrants, closedValues, reopenPreserved
       >>

GrantScopeAccess(p, s, access) ==
    /\ homeOpen
    /\ ownerWriter = "Owner"
    /\ access \in AccessLevels
    /\ durableGrants[<<p, s>>] = "None"
    /\ durableGrants' = [durableGrants EXCEPT ![<<p, s>>] = GrantStateForAccess(access)]
    /\ lastDelivery' = NoDelivery
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue,
           authStatus, watches, watchSeen, lastDelivered,
           closedGrants, closedValues, reopenPreserved
       >>

RevokeScopeAccess(p, s) ==
    /\ homeOpen
    /\ ownerWriter = "Owner"
    /\ HasReadGrant(durableGrants[<<p, s>>])
    /\ durableGrants' = [durableGrants EXCEPT ![<<p, s>>] = "Revoked"]
    /\ lastDelivery' = NoDelivery
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue,
           authStatus, watches, watchSeen, lastDelivered,
           closedGrants, closedValues, reopenPreserved
       >>

\* --- Collaborator remote operations (exact denial reasons + complete scoped rows) ---

RemoteReadOutcome(p, s, out, rows) ==
    LET grant == durableGrants[<<p, s>>]
        valid == authStatus[p] = "AuthValid"
    IN
    /\ IF ~homeOpen THEN
           /\ out = "HomeClosed"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED lastDelivered
       ELSE IF ~valid THEN
           /\ out = "AuthInvalidSignature"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED lastDelivered
       ELSE IF ~HasReadGrant(grant) THEN
           /\ out = "AuthzCapabilityDenied"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED lastDelivered
       ELSE
           /\ out = "Delivered"
           /\ rows = SingletonRow(s, scopeValue[s])
           /\ lastDelivered' = RecordDelivery(p, s, scopeValue[s])
           /\ lastDelivery' = DeliveryEvent(p, s, "Read", scopeValue[s])
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           authStatus, watches, watchSeen, closedGrants, closedValues, reopenPreserved
       >>

RemoteRead(p, s) ==
    \E out \in OutcomeKinds, rows \in {EmptyRows, SingletonRow(s, scopeValue[s])} :
        RemoteReadOutcome(p, s, out, rows)

RemoteWatchOpenOutcome(p, s, out, rows) ==
    LET grant == durableGrants[<<p, s>>]
        valid == authStatus[p] = "AuthValid"
    IN
    /\ IF ~homeOpen THEN
           /\ out = "HomeClosed"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED <<watches, watchSeen, lastDelivered>>
       ELSE IF ~valid THEN
           /\ out = "AuthInvalidSignature"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED <<watches, watchSeen, lastDelivered>>
       ELSE IF ~HasWatchGrant(grant) THEN
           /\ out = "AuthzCapabilityDenied"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED <<watches, watchSeen, lastDelivered>>
       ELSE
           /\ out = "Ok"
           /\ rows = SingletonRow(s, scopeValue[s])
           /\ watches' = [watches EXCEPT ![<<p, s>>] = "Open"]
           /\ watchSeen' = [watchSeen EXCEPT ![<<p, s>>] = scopeValue[s]]
           /\ lastDelivered' = RecordDelivery(p, s, scopeValue[s])
           /\ lastDelivery' = DeliveryEvent(p, s, "Watch", scopeValue[s])
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           authStatus, closedGrants, closedValues, reopenPreserved
       >>

RemoteWatchOpen(p, s) ==
    \E out \in OutcomeKinds, rows \in {EmptyRows, SingletonRow(s, scopeValue[s])} :
        RemoteWatchOpenOutcome(p, s, out, rows)

RemoteWatchPollOutcome(p, s, out, rows) ==
    LET grant == durableGrants[<<p, s>>]
        valid == authStatus[p] = "AuthValid"
    IN
    /\ IF ~homeOpen THEN
           /\ out = "HomeClosed"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED <<watches, watchSeen, lastDelivered>>
       ELSE IF ~valid THEN
           /\ out = "AuthInvalidSignature"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED <<watches, watchSeen, lastDelivered>>
       ELSE IF watches[<<p, s>>] /= "Open" THEN
           /\ out = "WatchNotFound"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED <<watches, watchSeen, lastDelivered>>
       ELSE IF ~HasWatchGrant(grant) THEN
           \* Fail closed immediately on revocation with exact WatchCapabilityRevoked reason.
           /\ out = "AuthzWatchCapabilityRevoked"
           /\ rows = EmptyRows
           /\ watches' = [watches EXCEPT ![<<p, s>>] = "Closed"]
           /\ watchSeen' = [watchSeen EXCEPT ![<<p, s>>] = 0]
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED lastDelivered
       ELSE IF scopeValue[s] > watchSeen[<<p, s>>] THEN
           /\ out = "Delivered"
           /\ rows = SingletonRow(s, scopeValue[s])
           /\ watchSeen' = [watchSeen EXCEPT ![<<p, s>>] = scopeValue[s]]
           /\ lastDelivered' = RecordDelivery(p, s, scopeValue[s])
           /\ lastDelivery' = DeliveryEvent(p, s, "Watch", scopeValue[s])
           /\ UNCHANGED watches
       ELSE
           /\ out = "Pending"
           /\ rows = EmptyRows
           /\ lastDelivery' = NoDelivery
           /\ UNCHANGED <<watches, watchSeen, lastDelivered>>
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           authStatus, closedGrants, closedValues, reopenPreserved
       >>

RemoteWatchPoll(p, s) ==
    \E out \in OutcomeKinds, rows \in {EmptyRows, SingletonRow(s, scopeValue[s])} :
        RemoteWatchPollOutcome(p, s, out, rows)

\* --- Directory Home lifecycle (single-owner close and reopen) ---

CloseHome ==
    /\ homeOpen
    /\ incarnation < MaxIncarnation
    /\ homeOpen' = FALSE
    /\ ownerWriter' = "None"
    /\ watches' = ClosedWatches
    /\ watchSeen' = ZeroPairVersions
    /\ closedGrants' = durableGrants
    /\ closedValues' = scopeValue
    /\ lastDelivery' = NoDelivery
    /\ UNCHANGED <<incarnation, scopeValue, durableGrants, authStatus, lastDelivered, reopenPreserved>>

ReopenHome ==
    /\ ~homeOpen
    /\ incarnation < MaxIncarnation
    /\ homeOpen' = TRUE
    /\ incarnation' = incarnation + 1
    /\ ownerWriter' = "Owner"
    /\ durableGrants' = closedGrants
    /\ scopeValue' = closedValues
    /\ reopenPreserved' = (closedGrants = durableGrants /\ closedValues = scopeValue)
    /\ closedGrants' = EmptyGrants
    /\ closedValues' = ZeroValues
    /\ watches' = ClosedWatches
    /\ watchSeen' = ZeroPairVersions
    /\ lastDelivery' = NoDelivery
    /\ UNCHANGED <<authStatus, lastDelivered>>

\* --- Adversarial token tampering and legitimate credential restoration ---

TamperAuthToken(p) ==
    /\ authStatus[p] = "AuthValid"
    /\ authStatus' = [authStatus EXCEPT ![p] = "AuthInvalid"]
    /\ lastDelivery' = NoDelivery
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           watches, watchSeen, lastDelivered, closedGrants, closedValues, reopenPreserved
       >>

RestoreAuthToken(p) ==
    /\ authStatus[p] = "AuthInvalid"
    /\ authStatus' = [authStatus EXCEPT ![p] = "AuthValid"]
    /\ lastDelivery' = NoDelivery
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           watches, watchSeen, lastDelivered, closedGrants, closedValues, reopenPreserved
       >>

Next ==
    \/ \E s \in Scopes : OwnerWrite(s)
    \/ \E p \in Principals, s \in Scopes, access \in AccessLevels : GrantScopeAccess(p, s, access)
    \/ \E p \in Principals, s \in Scopes : RevokeScopeAccess(p, s)
    \/ \E p \in Principals, s \in Scopes : RemoteRead(p, s)
    \/ \E p \in Principals, s \in Scopes : RemoteWatchOpen(p, s)
    \/ \E p \in Principals, s \in Scopes : RemoteWatchPoll(p, s)
    \/ CloseHome
    \/ ReopenHome
    \/ \E p \in Principals : TamperAuthToken(p)
    \/ \E p \in Principals : RestoreAuthToken(p)

Spec ==
    /\ Init
    /\ [][Next]_vars
    /\ WF_vars(RemoteRead("Alice", "Project1"))
    /\ WF_vars(RemoteWatchPoll("Alice", "Project1"))

\* ======================= Safety Invariants =======================

NeverLeakUngrantedScope ==
    /\ \A p \in Principals, s \in Scopes :
        /\ (durableGrants[<<p, s>>] = "None") =>
               (lastDelivered[<<p, s>>] = 0)
        /\ (durableGrants[<<p, s>>] \in {"None", "ActiveReadOnly"}) =>
               (watches[<<p, s>>] = "Closed" /\ watchSeen[<<p, s>>] = 0)
    /\ lastDelivery.active =>
        /\ homeOpen
        /\ authStatus[lastDelivery.principal] = "AuthValid"
        /\ IF lastDelivery.kind = "Read"
           THEN HasReadGrant(durableGrants[<<lastDelivery.principal, lastDelivery.scope>>])
           ELSE HasWatchGrant(durableGrants[<<lastDelivery.principal, lastDelivery.scope>>])

NeverServeAfterRevoke ==
    \A p \in Principals, s \in Scopes :
        (durableGrants[<<p, s>>] = "Revoked" /\ watches[<<p, s>>] = "Closed") =>
            watchSeen[<<p, s>>] = 0

NeverServeAfterRevokeStep ==
    \A p \in Principals, s \in Scopes :
        (durableGrants[<<p, s>>] = "Revoked") =>
            /\ ~(lastDelivery'.active /\ lastDelivery'.principal = p /\ lastDelivery'.scope = s)
            /\ lastDelivered'[<<p, s>>] = lastDelivered[<<p, s>>]
            /\ watchSeen'[<<p, s>>] <= watchSeen[<<p, s>>]
            /\ (watches[<<p, s>>] = "Closed" => watches'[<<p, s>>] = "Closed")

NeverServeUnauthenticatedStep ==
    \A p \in Principals, s \in Scopes :
        (authStatus[p] = "AuthInvalid") =>
            /\ ~(lastDelivery'.active /\ lastDelivery'.principal = p /\ lastDelivery'.scope = s)
            /\ lastDelivered'[<<p, s>>] = lastDelivered[<<p, s>>]
            /\ watchSeen'[<<p, s>>] <= watchSeen[<<p, s>>]
            /\ (watches[<<p, s>>] = "Closed" => watches'[<<p, s>>] = "Closed")

GrantsDurableAcrossReopen ==
    /\ (~homeOpen) => (durableGrants = closedGrants /\ scopeValue = closedValues)
    /\ (~homeOpen) => (\A ps \in Principals \X Scopes : watches[ps] = "Closed")
    /\ reopenPreserved

NoConcurrentWriters ==
    /\ (homeOpen <=> ownerWriter = "Owner")
    /\ (~homeOpen <=> ownerWriter = "None")

NoMutationWhileClosedStep ==
    (~homeOpen /\ ~homeOpen') =>
        /\ scopeValue' = scopeValue
        /\ durableGrants' = durableGrants

\* ======================= Safety & Liveness Properties =======================

NeverServeAfterRevokeProp == [][NeverServeAfterRevokeStep]_vars
NeverServeUnauthenticatedProp == [][NeverServeUnauthenticatedStep]_vars
NoMutationWhileClosedProp == [][NoMutationWhileClosedStep]_vars

CollaboratorEventuallyObservesProject1Write ==
    \A v \in 1..MaxVersion :
        []( (homeOpen
             /\ scopeValue["Project1"] = v
             /\ authStatus["Alice"] = "AuthValid"
             /\ HasReadGrant(durableGrants[<<"Alice", "Project1">>]))
            => <>( lastDelivered[<<"Alice", "Project1">>] >= v
                   \/ ~homeOpen
                   \/ authStatus["Alice"] /= "AuthValid"
                   \/ ~HasReadGrant(durableGrants[<<"Alice", "Project1">>]) ) )

WatchEventuallyDeliversProject1Write ==
    \A v \in 1..MaxVersion :
        []( (homeOpen
             /\ scopeValue["Project1"] = v
             /\ authStatus["Alice"] = "AuthValid"
             /\ HasWatchGrant(durableGrants[<<"Alice", "Project1">>])
             /\ watches[<<"Alice", "Project1">>] = "Open")
            => <>( watchSeen[<<"Alice", "Project1">>] >= v
                   \/ ~homeOpen
                   \/ authStatus["Alice"] /= "AuthValid"
                   \/ ~HasWatchGrant(durableGrants[<<"Alice", "Project1">>])
                   \/ watches[<<"Alice", "Project1">>] /= "Open" ) )

=============================================================================
