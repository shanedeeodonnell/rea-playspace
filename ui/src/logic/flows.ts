
// EDGE BUSINESS LOGIC

import { Edge, MarkerType } from "react-flow-renderer";
import getDataStore from "../data/DataStore";
import { DisplayEdge, DisplayNode } from "../data/models/Application/Display";
import { ObjectTypeMap } from "../data/models/ObjectTransformations";
import { getAlmostLastPart } from "../data/models/PathedData";
import { Action, Agent, ResourceSpecification, Unit } from "../data/models/Valueflows/Knowledge";
import { EconomicEvent } from "../data/models/Valueflows/Observation";
import { Commitment, Process } from "../data/models/Valueflows/Plan";
import { FlowShape, MeasurementShape } from "../types/valueflows";

/**
 * Given an ID, it returns the DisplayNode, it's associated Valueflows object and type.
 * This differs from Edges in that there is only one object associated with it, while
 * DisplayEdges have many objects associated with them.
 */
export const getDisplayNodeBy = (id: string): any => {
  const store = getDataStore();
  const displayNode: DisplayNode = store.getById(id);
  const vfType = getAlmostLastPart(displayNode.vfPath);
  const T = ObjectTypeMap[vfType];
  const vfNode: typeof T = store.getCursor(displayNode.vfPath);

  return {
    displayNode,
    vfType,
    vfNode
  }
}

/**
 * Returns the first flow associated with the edge, always returns the commitment if present
 */
export const getFirstCommitmentOrEvent = (displayEdge: DisplayEdge): FlowShape => {
  const store = getDataStore();
  let path: string;
  if (Array.isArray(displayEdge.vfPath)) {
    let flowIndex = 0;
    const commitmentIndex = displayEdge.vfPath.findIndex(
      (path) => getAlmostLastPart(path) === 'commitment'
    );
    if (commitmentIndex > 0) {
      flowIndex = commitmentIndex;
    }
    path = displayEdge.vfPath[flowIndex];
  } else {
    path = displayEdge.vfPath;
  }
  return store.getCursor(path);
}

/**
 * Returns the Commitment and Events associated with a display edge in a structured fashion.
 * @param vfPath 
 * @returns 
 */
export const getCommitmentAndEvents = (vfPath: string[] | string): {commitment: Commitment, events: EconomicEvent[]} => {
  const store = getDataStore();
  const events = new Array<EconomicEvent>();
  let commitmentPath: string;
  if (Array.isArray(vfPath)) {
    vfPath.forEach(
      (flowPath) => {
        if (getAlmostLastPart(flowPath) === 'commitment') {
          commitmentPath = flowPath;
        } else {
          events.push(store.getCursor(flowPath) as EconomicEvent);
        }
      }
    );
  } else {
    commitmentPath = vfPath;
  }
  const commitment = commitmentPath ? store.getCursor(commitmentPath) as Commitment: null;
  return {commitment, events};
}

/**
 * Returns a set of sensible defaults for a flow
 */
export const flowDefaults = {
  // This is an input
  'resourceSpecification-process': (planId: string, resource: ResourceSpecification, process: Process): FlowShape => {
    return {
      plannedWithin: planId,
      resourceConformsTo: resource.id,
      provider: null,
      receiver: process.inScopeOf,
      action: 'use',
      inputOf: process.id
    };
  },
  // this is an output
  'process-resourceSpecification': (planId: string, process: Process, resource: ResourceSpecification): FlowShape => {
    return {
      plannedWithin: planId,
      resourceConformsTo: resource.id,
      provider: process.inScopeOf,
      receiver: null,
      action: 'use',
      outputOf: process.id
    };
  },
  // This is a transfer, set up the flow between the agents. User must select a
  // resourceSpecification.
  // TODO: The data model doesn't actualy have all the data needed to reconstruct
  // the graph. We don't actually know what the resource this is being transfered
  // to is without logging an event with resourceInventoriedAs and toResourceInventoriedAs
  'resourceSpecification-resourceSpecification': (planId: string, source: ResourceSpecification, target: ResourceSpecification): FlowShape => {
    return {
      plannedWithin: planId,
      resourceConformsTo: source.id,
      provider: null,
      receiver: null,
      action: 'transfer'
    };
  }
}

/**
 * Returns the minimum necessary changes to a flow when an edge is rewired.
 */
export const flowUpdates = {
  // This is an input
  'resourceSpecification-process': (flow: FlowShape, resource: ResourceSpecification, process: Process): FlowShape => {
    const clone = {...flow};
    clone.resourceConformsTo = resource.id;
    clone.receiver = process.inScopeOf;
    clone.inputOf = process.id;
    return clone;
  },
  // this is an output
  'process-resourceSpecification': (flow: FlowShape, process: Process, resource: ResourceSpecification): FlowShape => {
    const clone = {...flow};
    clone.resourceConformsTo = resource.id;
    clone.provider = process.inScopeOf;
    clone.outputOf = process.id;
    return clone;
  },
  // This is a transfer, set up the flow between the agents. User must select a resourceSpecification.
  'resourceSpecification-resourceSpecification': (flow: FlowShape, source: ResourceSpecification, target: ResourceSpecification): FlowShape => {
    const clone = {...flow};
    clone.resourceConformsTo = source.id;
    clone.action = flow.action in ['transfer', 'transfer-all-rights', 'transfer-custody'] ? flow.action : 'transfer';
    clone.inputOf = '';
    clone.outputOf = '';
    return clone;
  }
}

/**
 * Validate if a connection can happen
 */
export const validateFlow = (sourceType: string, targetType: string): boolean => {
  const validSourceTargets = {
    'resourceSpecification': [
      'process',
      'resourceSpecification'
    ],
    'process': [
      'resourceSpecification'
    ]
  };
  return validSourceTargets[sourceType]?.indexOf(targetType) >= 0
}

/**
 * Returns a label for a flow (a Commitment or EconomicEvent)
 */
export const getLabelForFlow = (flow: FlowShape, provider: Agent, receiver: Agent, actions: Action[], units: Unit[]): string => {
  const action: Action = actions.find((action) => action.id == flow.action);
  const measurement: MeasurementShape = {
    hasNumericalValue: 0,
    hasUnit: ''
  };

  if (flow.resourceQuantity != null && flow.effortQuantity == null) {
    measurement.hasNumericalValue = flow.resourceQuantity.hasNumericalValue;
    measurement.hasUnit = flow.resourceQuantity.hasUnit;
  }
  if (flow.resourceQuantity == null && flow.effortQuantity != null) {
    measurement.hasNumericalValue = flow.effortQuantity.hasNumericalValue;
    measurement.hasUnit = flow.effortQuantity.hasUnit;
  }

  const unit: Unit = units.find((unit) => unit.id == measurement.hasUnit);
  
  const baseLabel = `${action.label} ${measurement.hasNumericalValue} ${unit.symbol}`

  if (flow.inputOf && flow.provider) {
    return `${provider.name}: ${baseLabel}`;
  } else if (flow.outputOf && flow.receiver) {
    return `${baseLabel} for ${receiver.name}`;
  } else if (flow.provider && flow.receiver) {
    return `${baseLabel} from ${provider.name} to ${receiver.name}`;
  }
  return baseLabel;
}

/**
 * Function to generate the label for a commitment edge.
 */
 export const getLabelForDisplayEdge = (displayEdge: DisplayEdge): string => {
  const store = getDataStore();
  try {
    const flow: FlowShape = getFirstCommitmentOrEvent(displayEdge);
    const actions: Action[] = store.getActions();
    const units: Unit[] = store.getUnits();
    const provider = store.getById(flow.provider);
    const receiver = store.getById(flow.receiver);
    return getLabelForFlow(flow, provider, receiver, actions, units);
  } catch(err) {
    console.error(err);
    return "placeholder";
  }
}

/**
 * This returns the data in the format React Flow requires. The data.id is used
 * to map back to the DisplayEdge object in the store.
 *
 * TODO: maybe submit a PR to React Flows to either export the function that
 * builds these IDs, or make it more amenable to unique external IDs.
 */
export const displayEdgeToEdge = (displayEdge: DisplayEdge): Edge  => {
  return {
    id: `reactflow__edge-${displayEdge.source}${displayEdge.sourceHandle || ''}-${displayEdge.target}${displayEdge.targetHandle || ''}`,
    source: displayEdge.source,
    target: displayEdge.target,
    sourceHandle: displayEdge.sourceHandle,
    targetHandle: displayEdge.targetHandle,
    label: getLabelForDisplayEdge(displayEdge),
    labelBgStyle: { fill: '#fff', color: '#fff', fillOpacity: 0.7 },
    markerEnd: {
      type: MarkerType.ArrowClosed,
    },
    data: {
      id: displayEdge.id
    }
  }
}
