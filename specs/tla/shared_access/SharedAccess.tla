---------------------------- MODULE SharedAccess ----------------------------
EXTENDS Integers, Sequences, FiniteSets

(*
  Formal specification of WORTH Shared Access (Daymark Wave 4, P4.2 / G11):
  Single-owner Directory Home with durable WORTH-governed scope capability
  grants, fail-closed remote read/watch delegation, constant-time authenticated
  sessions, immediate revocation, and Directory Home close/reopen durability.

  Safety invariants & action properties:
    - NeverLeakUngrantedScope (state invariant + step property)
    - NeverServeAfterRevoke (step property + state witness)
    - NeverServeUnauthenticated (step property + state witness)
    - GrantsDurableAcrossReopen (state invariant)
    - NoConcurrentWriters (state invariant + step property)

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
    watches,            \* [Principals \X Scopes -> {"Closed", "Open"}]: active remote watches
    watchSeen,          \* [Principals \X Scopes -> 0..MaxVersion]: last version delivered on open watch (0 when closed)
    aliceP1Observed,    \* 0..MaxVersion: latest Project1 version observed by Alice (for liveness)
    closedGrants,       \* durableGrants snapshot while closed (EmptyGrants while open)
    closedValues,       \* scopeValue snapshot while closed (ZeroValues while open)
    reopenPreserved     \* BOOLEAN: TRUE iff latest ReopenHome preserved durableGrants and scopeValue exactly

vars == <<
    homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
    authStatus, watches, watchSeen, aliceP1Observed, closedGrants,
    closedValues, reopenPreserved
>>

GrantStates == {"None", "ActiveReadOnly", "ActiveReadAndWatch", "Revoked"}
AccessLevels == {"ReadOnly", "ReadAndWatch"}
AuthStates == {"AuthValid", "AuthInvalid"}
WatchStates == {"Closed", "Open"}

EmptyGrants == [ps \in Principals \X Scopes |-> "None"]
ZeroValues == [s \in Scopes |-> 0]
ClosedWatches == [ps \in Principals \X Scopes |-> "Closed"]
ZeroWatchSeen == [ps \in Principals \X Scopes |-> 0]

OutcomeKinds == {
    "Ok",
    "Delivered",
    "Pending",
    "AuthenticationRefused",
    "AuthorizationRefused",
    "WatchRefused",
    "HomeClosed"
}

HasReadGrant(g) == g \in {"ActiveReadOnly", "ActiveReadAndWatch"}
HasWatchGrant(g) == g = "ActiveReadAndWatch"

GrantStateForAccess(access) ==
    IF access = "ReadOnly" THEN "ActiveReadOnly" ELSE "ActiveReadAndWatch"

TypeOK ==
    /\ homeOpen \in BOOLEAN
    /\ incarnation \in 1..MaxIncarnation
    /\ ownerWriter \in {"Owner", "None"}
    /\ scopeValue \in [Scopes -> 0..MaxVersion]
    /\ durableGrants \in [Principals \X Scopes -> GrantStates]
    /\ authStatus \in [Principals -> AuthStates]
    /\ watches \in [Principals \X Scopes -> WatchStates]
    /\ watchSeen \in [Principals \X Scopes -> 0..MaxVersion]
    /\ aliceP1Observed \in 0..MaxVersion
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
    /\ watchSeen = ZeroWatchSeen
    /\ aliceP1Observed = 0
    /\ closedGrants = EmptyGrants
    /\ closedValues = ZeroValues
    /\ reopenPreserved = TRUE

\* --- Owner-only mutations (single-owner Directory Home invariant) ---

OwnerWrite(s) ==
    /\ homeOpen
    /\ ownerWriter = "Owner"
    /\ scopeValue[s] < MaxVersion
    /\ scopeValue' = [scopeValue EXCEPT ![s] = @ + 1]
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, durableGrants,
           authStatus, watches, watchSeen, aliceP1Observed,
           closedGrants, closedValues, reopenPreserved
       >>

GrantScopeAccess(p, s, access) ==
    /\ homeOpen
    /\ ownerWriter = "Owner"
    /\ access \in AccessLevels
    /\ durableGrants[<<p, s>>] = "None"
    /\ durableGrants' = [durableGrants EXCEPT ![<<p, s>>] = GrantStateForAccess(access)]
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue,
           authStatus, watches, watchSeen, aliceP1Observed,
           closedGrants, closedValues, reopenPreserved
       >>

RevokeScopeAccess(p, s) ==
    /\ homeOpen
    /\ ownerWriter = "Owner"
    /\ HasReadGrant(durableGrants[<<p, s>>])
    /\ durableGrants' = [durableGrants EXCEPT ![<<p, s>>] = "Revoked"]
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue,
           authStatus, watches, watchSeen, aliceP1Observed,
           closedGrants, closedValues, reopenPreserved
       >>

\* --- Collaborator remote operations (always authenticated + WORTH-governed) ---

RemoteReadOutcome(p, s, out, val) ==
    LET grant == durableGrants[<<p, s>>]
        valid == authStatus[p] = "AuthValid"
    IN
    /\ IF ~homeOpen THEN
           /\ out = "HomeClosed"
           /\ val = 0
           /\ UNCHANGED aliceP1Observed
       ELSE IF ~valid THEN
           /\ out = "AuthenticationRefused"
           /\ val = 0
           /\ UNCHANGED aliceP1Observed
       ELSE IF ~HasReadGrant(grant) THEN
           /\ out = "AuthorizationRefused"
           /\ val = 0
           /\ UNCHANGED aliceP1Observed
       ELSE
           /\ out = "Delivered"
           /\ val = scopeValue[s]
           /\ aliceP1Observed' =
                  IF p = "Alice" /\ s = "Project1" /\ scopeValue[s] > aliceP1Observed
                  THEN scopeValue[s]
                  ELSE aliceP1Observed
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           authStatus, watches, watchSeen, closedGrants, closedValues, reopenPreserved
       >>

RemoteRead(p, s) ==
    \E out \in OutcomeKinds, val \in 0..MaxVersion : RemoteReadOutcome(p, s, out, val)

RemoteWatchOpenOutcome(p, s, out, val) ==
    LET grant == durableGrants[<<p, s>>]
        valid == authStatus[p] = "AuthValid"
    IN
    /\ IF ~homeOpen THEN
           /\ out = "HomeClosed"
           /\ val = 0
           /\ UNCHANGED <<watches, watchSeen, aliceP1Observed>>
       ELSE IF ~valid THEN
           /\ out = "AuthenticationRefused"
           /\ val = 0
           /\ UNCHANGED <<watches, watchSeen, aliceP1Observed>>
       ELSE IF ~HasWatchGrant(grant) THEN
           /\ out = "AuthorizationRefused"
           /\ val = 0
           /\ UNCHANGED <<watches, watchSeen, aliceP1Observed>>
       ELSE
           /\ out = "Ok"
           /\ val = scopeValue[s]
           /\ watches' = [watches EXCEPT ![<<p, s>>] = "Open"]
           /\ watchSeen' = [watchSeen EXCEPT ![<<p, s>>] = scopeValue[s]]
           /\ aliceP1Observed' =
                  IF p = "Alice" /\ s = "Project1" /\ scopeValue[s] > aliceP1Observed
                  THEN scopeValue[s]
                  ELSE aliceP1Observed
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           authStatus, closedGrants, closedValues, reopenPreserved
       >>

RemoteWatchOpen(p, s) ==
    \E out \in OutcomeKinds, val \in 0..MaxVersion : RemoteWatchOpenOutcome(p, s, out, val)

RemoteWatchPollOutcome(p, s, out, val) ==
    LET grant == durableGrants[<<p, s>>]
        valid == authStatus[p] = "AuthValid"
    IN
    /\ IF ~homeOpen THEN
           /\ out = "HomeClosed"
           /\ val = 0
           /\ UNCHANGED <<watches, watchSeen, aliceP1Observed>>
       ELSE IF ~valid THEN
           /\ out = "AuthenticationRefused"
           /\ val = 0
           /\ UNCHANGED <<watches, watchSeen, aliceP1Observed>>
       ELSE IF watches[<<p, s>>] /= "Open" THEN
           /\ out = "WatchRefused"
           /\ val = 0
           /\ UNCHANGED <<watches, watchSeen, aliceP1Observed>>
       ELSE IF ~HasWatchGrant(grant) THEN
           \* Fail closed immediately on revocation and terminate the watch handle.
           /\ out = "AuthorizationRefused"
           /\ val = 0
           /\ watches' = [watches EXCEPT ![<<p, s>>] = "Closed"]
           /\ watchSeen' = [watchSeen EXCEPT ![<<p, s>>] = 0]
           /\ UNCHANGED aliceP1Observed
       ELSE IF scopeValue[s] > watchSeen[<<p, s>>] THEN
           /\ out = "Delivered"
           /\ val = scopeValue[s]
           /\ watchSeen' = [watchSeen EXCEPT ![<<p, s>>] = scopeValue[s]]
           /\ aliceP1Observed' =
                  IF p = "Alice" /\ s = "Project1" /\ scopeValue[s] > aliceP1Observed
                  THEN scopeValue[s]
                  ELSE aliceP1Observed
           /\ UNCHANGED watches
       ELSE
           /\ out = "Pending"
           /\ val = 0
           /\ UNCHANGED <<watches, watchSeen, aliceP1Observed>>
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           authStatus, closedGrants, closedValues, reopenPreserved
       >>

RemoteWatchPoll(p, s) ==
    \E out \in OutcomeKinds, val \in 0..MaxVersion : RemoteWatchPollOutcome(p, s, out, val)

\* --- Directory Home lifecycle (single-owner close and reopen) ---

CloseHome ==
    /\ homeOpen
    /\ incarnation < MaxIncarnation
    /\ homeOpen' = FALSE
    /\ ownerWriter' = "None"
    /\ watches' = ClosedWatches
    /\ watchSeen' = ZeroWatchSeen
    /\ closedGrants' = durableGrants
    /\ closedValues' = scopeValue
    /\ UNCHANGED <<incarnation, scopeValue, durableGrants, authStatus, aliceP1Observed, reopenPreserved>>

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
    /\ watchSeen' = ZeroWatchSeen
    /\ UNCHANGED <<authStatus, aliceP1Observed>>

\* --- Adversarial token tampering and legitimate credential restoration ---

TamperAuthToken(p) ==
    /\ authStatus[p] = "AuthValid"
    /\ authStatus' = [authStatus EXCEPT ![p] = "AuthInvalid"]
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           watches, watchSeen, aliceP1Observed, closedGrants, closedValues, reopenPreserved
       >>

RestoreAuthToken(p) ==
    /\ authStatus[p] = "AuthInvalid"
    /\ authStatus' = [authStatus EXCEPT ![p] = "AuthValid"]
    /\ UNCHANGED <<
           homeOpen, incarnation, ownerWriter, scopeValue, durableGrants,
           watches, watchSeen, aliceP1Observed, closedGrants, closedValues, reopenPreserved
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
    \A p \in Principals, s \in Scopes :
        (durableGrants[<<p, s>>] \in {"None", "ActiveReadOnly"}) =>
            (watches[<<p, s>>] = "Closed" /\ watchSeen[<<p, s>>] = 0)

NeverServeAfterRevoke ==
    \A p \in Principals, s \in Scopes :
        (durableGrants[<<p, s>>] = "Revoked" /\ watches[<<p, s>>] = "Closed") =>
            watchSeen[<<p, s>>] = 0

NeverServeAfterRevokeStep ==
    \A p \in Principals, s \in Scopes :
        (durableGrants[<<p, s>>] = "Revoked") =>
            /\ watchSeen'[<<p, s>>] <= watchSeen[<<p, s>>]
            /\ (watches[<<p, s>>] = "Closed" => watches'[<<p, s>>] = "Closed")
            /\ ((p = "Alice" /\ s = "Project1") => aliceP1Observed' = aliceP1Observed)

NeverServeUnauthenticatedStep ==
    \A p \in Principals, s \in Scopes :
        (authStatus[p] = "AuthInvalid") =>
            /\ (watches[<<p, s>>] = "Closed" => watches'[<<p, s>>] = "Closed")
            /\ watchSeen'[<<p, s>>] <= watchSeen[<<p, s>>]
            /\ ((p = "Alice" /\ s = "Project1") => aliceP1Observed' = aliceP1Observed)

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
            => <>( aliceP1Observed >= v
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
