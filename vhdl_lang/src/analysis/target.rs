use super::analyze::*;
use super::names::*;
use super::region::*;
use crate::ast::*;
use crate::data::*;

/// Analysis of assignment targets
///
/// examples:
///   target <= 1;
///   target(0).elem := 1

impl<'a> AnalyzeContext<'a> {
    pub fn resolve_target(
        &self,
        scope: &Scope<'_>,
        target: &mut WithPos<Target>,
        assignment_type: AssignmentType,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> FatalResult<Option<TypeEnt>> {
        match target.item {
            Target::Name(ref mut name) => {
                self.resolve_target_name(scope, name, &target.pos, assignment_type, diagnostics)
            }
            Target::Aggregate(ref mut assocs) => {
                self.analyze_aggregate(scope, assocs, diagnostics)?;
                Ok(None)
            }
        }
    }

    pub fn resolve_target_name(
        &self,
        scope: &Scope<'_>,
        target: &mut Name,
        target_pos: &SrcPos,
        assignment_type: AssignmentType,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> FatalResult<Option<TypeEnt>> {
        match self.resolve_object_prefix(
            scope,
            target_pos,
            target,
            "Invalid assignment target",
            diagnostics,
        ) {
            Ok(Some(resolved_name)) => {
                if let ResolvedName::ObjectSelection {
                    ref base,
                    ref type_mark,
                } = resolved_name
                {
                    if !is_valid_assignment_target(base) {
                        diagnostics.push(Diagnostic::error(
                            target_pos,
                            format!(
                                "{} may not be the target of an assignment",
                                base.describe_class()
                            ),
                        ));
                    } else if !is_valid_assignment_type(base, assignment_type) {
                        diagnostics.push(Diagnostic::error(
                            target_pos,
                            format!(
                                "{} may not be the target of a {} assignment",
                                base.describe_class(),
                                assignment_type.to_str()
                            ),
                        ));
                    }
                    Ok(Some(type_mark.clone()))
                } else {
                    diagnostics.push(Diagnostic::error(target_pos, "Invalid assignment target"));
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(err) => {
                err.add_to(diagnostics)?;
                Ok(None)
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum AssignmentType {
    // Assignement with <=
    Signal,
    // Assignment with :=
    Variable,
}

impl AssignmentType {
    fn to_str(self) -> &'static str {
        match self {
            AssignmentType::Signal => "signal",
            AssignmentType::Variable => "variable",
        }
    }
}

/// Check that the assignment target is a writable object and not constant or input only
fn is_valid_assignment_target(base: &ObjectBase) -> bool {
    base.class() != ObjectClass::Constant && !matches!(base.mode(), Some(Mode::In))
}

// Check that a signal is not the target of a variable assignment and vice-versa
fn is_valid_assignment_type(base: &ObjectBase, assignment_type: AssignmentType) -> bool {
    let class = base.class();
    match assignment_type {
        AssignmentType::Signal => matches!(class, ObjectClass::Signal),
        AssignmentType::Variable => {
            matches!(class, ObjectClass::Variable | ObjectClass::SharedVariable)
        }
    }
}
