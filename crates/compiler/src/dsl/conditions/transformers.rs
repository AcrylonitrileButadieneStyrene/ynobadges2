use crate::{dsl::conditions::parser::DelayTarget, format::output::ConditionTrigger};

pub fn push_switch(parser: &mut super::parser::Parser, id: u16, value: bool) {
    if let (Some(switch_ids), Some(switch_values)) = (
        &mut parser.condition.switch_ids,
        &mut parser.condition.switch_values,
    ) {
        switch_ids.push(id);
        switch_values.push(value);
    } else if let Some(existing) = parser.condition.switch_id {
        parser.condition.switch_ids = Some(vec![existing, id]);
        parser.condition.switch_values = Some(vec![parser.condition.switch_value, value]);
        parser.condition.switch_id = None;
        parser.condition.switch_value = false;
    } else {
        parser.condition.switch_id = Some(id);
        parser.condition.switch_value = value;
    }

    parser.state.last_delayable = Some(DelayTarget::Switch);
    parser.state.has_trigger = true;
    if parser.condition.trigger == Some(ConditionTrigger::Coords) {
        parser.condition.trigger = None;
    }
}

pub fn push_variable(parser: &mut super::parser::Parser, id: u16, op: String, value: i32) {
    if let (Some(var_ids), Some(var_ops), Some(var_values)) = (
        &mut parser.condition.var_ids,
        &mut parser.condition.var_ops,
        &mut parser.condition.var_values,
    ) {
        var_ids.push(id);
        var_ops.push(op);
        var_values.push(value);
    } else if let (Some(existing_id), Some(existing_value)) =
        (parser.condition.var_id, parser.condition.var_value)
    {
        let existing_op = parser
            .condition
            .var_op
            .clone()
            .unwrap_or_else(|| "=".to_string());

        parser.condition.var_ids = Some(vec![existing_id, id]);
        parser.condition.var_ops = Some(vec![existing_op.clone(), op]);
        parser.condition.var_values = Some(vec![existing_value, value]);

        parser.condition.var_id = None;
        parser.condition.var_op = None;
        parser.condition.var_value = None;
    } else {
        parser.condition.var_id = Some(id);
        if op != "=" {
            parser.condition.var_op = Some(op);
        }
        parser.condition.var_value = Some(value);
    }

    parser.state.last_delayable = Some(DelayTarget::Variable);
    parser.state.has_trigger = true;
    if parser.condition.trigger == Some(ConditionTrigger::Coords) {
        parser.condition.trigger = None;
    }
}
