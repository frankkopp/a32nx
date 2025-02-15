use super::{
    avionics_full_duplex_switch::AvionicsFullDuplexSwitch,
    AvionicsDataCommunicationNetworkEndpoint, AvionicsDataCommunicationNetworkMessageData,
    AvionicsDataCommunicationNetworkMessageIdentifier,
};
use crate::{
    shared::{ElectricalBusType, ElectricalBuses},
    simulation::{
        InitContext, Read, SimulationElement, SimulatorReader, SimulatorWriter, VariableIdentifier,
        Write,
    },
};
use std::{cell::RefCell, rc::Rc};

pub struct CoreProcessingInputOutputModule {
    power_supply: ElectricalBusType,
    is_powered: bool,
    available_id: VariableIdentifier,
    failure_indication_id: VariableIdentifier,
    failure_indication: bool,
    connected_switches: Vec<Rc<RefCell<AvionicsFullDuplexSwitch>>>,
}

impl CoreProcessingInputOutputModule {
    pub fn new(
        context: &mut InitContext,
        name: &str,
        power_supply: ElectricalBusType,
        connected_switches: Vec<Rc<RefCell<AvionicsFullDuplexSwitch>>>,
    ) -> Self {
        Self {
            power_supply,
            is_powered: false,
            available_id: context.get_identifier(format!("CPIOM_{}_AVAIL", name)),
            failure_indication_id: context.get_identifier(format!("CPIOM_{}_FAILURE", name)),
            failure_indication: false,
            connected_switches,
        }
    }

    pub fn is_available(&self) -> bool {
        self.is_powered & !self.failure_indication
    }
}

impl AvionicsDataCommunicationNetworkEndpoint for CoreProcessingInputOutputModule {
    fn recv_value(
        &self,
        id: &AvionicsDataCommunicationNetworkMessageIdentifier,
    ) -> Option<AvionicsDataCommunicationNetworkMessageData> {
        // TODO: check if there is a newer message on the other networks
        self.connected_switches
            .iter()
            .find_map(|switch| switch.borrow().recv_value(id))
    }

    fn send_value(
        &self,
        id: &AvionicsDataCommunicationNetworkMessageIdentifier,
        value: AvionicsDataCommunicationNetworkMessageData,
    ) {
        for switch in &self.connected_switches {
            switch.borrow_mut().send_value(id, value.clone());
        }
    }
}

impl SimulationElement for CoreProcessingInputOutputModule {
    fn read(&mut self, reader: &mut SimulatorReader) {
        self.failure_indication = reader.read(&self.failure_indication_id);
    }

    fn write(&self, writer: &mut SimulatorWriter) {
        writer.write(&self.available_id, self.is_available());
    }

    fn receive_power(&mut self, buses: &impl ElectricalBuses) {
        self.is_powered = buses.is_powered(self.power_supply);
    }
}
