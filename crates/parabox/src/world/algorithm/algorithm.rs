use super::cycle::Cycle;
use super::movement::{
    Direction, EatInfo, EnterInfo, ExitInfo, IntoMoveResult, MoveProcessor, MoveResult, Movement,
    SourceArrow, TargetArrow,
};
use super::rational::Rational;
use crate::world::query::PositionState;
use crate::{BlockKey, Position, World};
use parabox_macros::trace_func;
use std::collections::HashSet;
use tracing::{debug, instrument};

pub(crate) struct Algorithm {
    trace: Cycle<BlockKey>,
    movements: Vec<Movement>,
    positioned: HashSet<Position>,
}

impl Algorithm {
    pub fn new() -> Self {
        Self {
            trace: Cycle::new(),
            movements: Vec::new(),
            positioned: HashSet::new(),
        }
    }
}

impl Algorithm {
    /// Returns `Ok(TargetArrow)` if the target is inside the container, or
    /// `Err(ExitInfo)` if the target is out of bounds.
    #[trace_func]
    #[instrument(skip(self, world))]
    fn source_to_target(
        &self,
        world: &World,
        source: SourceArrow,
    ) -> Result<TargetArrow, ExitInfo> {
        // Compute the target pos according to the source pos and direction.
        let (x, y) = source.position.pos;
        let (x, y) = (x as isize, y as isize);
        let (dx, dy) = source.direction.delta();
        let (x, y) = (x + dx, y + dy);

        let container = source.position.container.unwrap();
        let (width, height) = container.get(world).proto.size();

        // Check if the target pos is out of bounds.
        if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
            // Compute and return the exit info.
            let (offset, total) = match source.direction {
                Direction::North | Direction::South => (x as usize, width),
                Direction::East | Direction::West => (y as usize, height),
            };

            let precise = (source.precise + offset) / total;

            Err(ExitInfo {
                from: container,
                direction: source.direction,
                precise,
            })
        } else {
            // Return the target arrow.
            Ok(TargetArrow::new(
                Position::inside(container, (x as usize, y as usize)),
                source.direction,
                source.precise,
            ))
        }
    }

    /// If the target position is empty, which means the movement can be
    /// directly executed, then `Ok(Movement)` will be returned.
    ///
    /// If the target position is taken by a block, then `Err((Movement,
    /// EnterInfo))` is returned, where `EnterInfo` is the information about the
    /// possible block entering.
    fn target_to_movement(
        &self,
        world: &World,
        key: BlockKey,
        target: TargetArrow,
    ) -> Result<Movement, (Movement, EnterInfo)> {
        let movement = Movement::new(key, target.position);

        match world.position_state(target.position) {
            PositionState::Present(key) => Err((
                movement,
                EnterInfo {
                    into: key,
                    direction: target.direction,
                    precise: target.precise,
                },
            )),
            PositionState::Empty => Ok(movement),
            _ => unreachable!("Target position is not valid"),
        }
    }

    /// Confirms the movement, also returning `Ok(true)` for convenience.
    ///
    /// If `cycling` is `true`, then the movement will be checked before being
    /// saved. This is because if the movement occurs in a cycle, a block
    /// that triggers the cycle does not necessarily move.
    #[trace_func]
    #[instrument(skip(self))]
    fn confirm(&mut self, movement: Movement, cycling: bool) -> MoveResult<bool> {
        // Check if the movement is blocked.
        //
        // When `cycling` is `true`, the target block is always moving, leaving an empty
        // space there. It will be blocked only if another block in the cycle is trying
        // to move there.
        //
        // Since this function is called in the recursion of pushing movements, in the
        // reverse order of the cycle, later movements will always be confirmed first,
        // so finally the confirmed movements will be exactly the loop in the pushing
        // cycle.
        let blocked = cycling && self.positioned.contains(&movement.target);
        debug!("whether blocked: {}", blocked);

        if !blocked {
            // Save the movement.
            self.movements.push(movement);
            self.positioned.insert(movement.target);
        }

        Ok(true)
    }
}

impl Algorithm {
    /// Tries to push the blocks from the source position.
    ///
    /// This function will evaluate the exit movements. Then it will call
    /// [Algorithm::push_into] for the rest evaluation.
    ///
    /// Exits from a [crate::ProtoType::Void] block will be forbidden.
    #[trace_func]
    #[instrument(skip(self, world))]
    fn push_from(&mut self, world: &World, key: BlockKey, source: SourceArrow) -> MoveResult<bool> {
        let mut cycle: Cycle<BlockKey, Rational> = Cycle::new();
        let mut current: SourceArrow = source;

        loop {
            // Try to push the block into the target position.
            match self.source_to_target(world, current) {
                Ok(target) => {
                    // The target is inside the container.
                    // Push the block into the target position.
                    break self.push_into(world, key, target, false);
                }
                Err(mut info) => {
                    // The target is out of bounds.
                    // Try to resolve the exit info.
                    while let Some(original) = cycle.push(info.from, info.precise) {
                        // The exit info is in a cycle.
                        // Use infinity to resolve the cycle.
                        info = MoveProcessor.infinity(
                            world,
                            ExitInfo {
                                from: info.from,
                                direction: info.direction,
                                precise: *original,
                            },
                        )?;
                    }

                    // Forbid exits from a void block.
                    if info.from.is_void(world) {
                        break Ok(false);
                    }

                    current = MoveProcessor.exit(world, info)?;
                }
            }
        }
    }

    /// Tries to push the blocks into the target position.
    ///
    /// This function will evaluate the pushing, entering and eating movements.
    /// If the target is empty, the movement will be directly confirmed.
    /// Otherwise, the source block will first try to enter the target block,
    /// and then try to eat the target block. When multiple entering movements
    /// happen, eating movements will be tried in the reverse order.
    ///
    /// `eating` is `true` if the function is called during an eating movement.
    /// When `A` eats `B`, it is equivalent to `B` entering `A`, which is almost
    /// equivalent to pushing `B` into the position of `A`, except that `A`
    /// should not try to push `B` or eat `B`.
    ///
    /// Movements directly inside a [crate::ProtoType::Void] block will be
    /// forbidden.
    #[trace_func]
    #[instrument(skip(self, world))]
    fn push_into(
        &mut self,
        world: &World,
        key: BlockKey,
        target: TargetArrow,
        eating: bool,
    ) -> MoveResult<bool> {
        if self.trace.push(key, ()).is_some() {
            // Found a pushing cycle.
            debug!("cycle: {:?}", self.trace);

            // Resolve the cycle by confirming the movements in the cycle.
            //
            // To achieve this, we start by returning `Ok(true)` for the last block in the
            // cycle. Then every block in the cycle will confirm its movement by recursion.
            return Ok(true);
        }

        let mut cycle: Cycle<EnterInfo> = Cycle::new();
        let mut current: TargetArrow = target;

        // Stop the first pushing try when eating.
        let mut can_push = !eating;

        loop {
            // Try to push the block into the target position.
            match self.target_to_movement(world, key, current) {
                Ok(movement) => {
                    // The target is empty.
                    // Confirm the movement.
                    return self.confirm(movement, false);
                }
                Err((movement, mut info)) => {
                    // The target is taken by a block.
                    // Try to push the target block.
                    // But do not push when eating.
                    if can_push && self.push(world, info.into, info.direction)? {
                        return self.confirm(movement, true);
                    }

                    can_push = true;

                    // Forbid entering movements directly inside a void block.
                    if movement.target.is_in_void(world) {
                        break;
                    }

                    // Resolve the transferred enter info when entering an alias block.
                    info = MoveProcessor.alias(world, info);

                    // Try to resolve the enter info.
                    while cycle.push(info, ()).is_some() {
                        // The enter info is in a cycle.
                        // Use epsilon to resolve the cycle.
                        info = MoveProcessor.epsilon(world, info)?;
                    }

                    match MoveProcessor.enter(world, info) {
                        // Successfully entered the target block.
                        // Go to the next push-enter loop.
                        Some(next) => current = next,
                        // Failed to enter the target block.
                        None => break,
                    }
                }
            }
        }

        // Try to eat the target block, in the reverse order of entering movements.
        while let Some((info, _)) = cycle.pop() {
            // Stop the last eating try when eating.
            // The last eating try corresponds to the first entering movement.
            if eating && cycle.is_empty() {
                break;
            }

            let eat_info = EatInfo {
                eat: key,
                ate: info.into,
                direction: info.direction,
            };

            // Cannot eat a static block.
            if eat_info.ate.get(world).proto.is_static() {
                continue;
            }

            // Convert the eating movement into an entering movement.
            let enter_info = MoveProcessor.eat(world, eat_info);

            let enter_target = TargetArrow::new(
                enter_info.into.get(world).state.position,
                enter_info.direction,
                enter_info.precise,
            );

            if self.push_into(world, eat_info.ate, enter_target, true)? {
                return self.confirm(
                    Movement::new(key, info.into.get(world).state.position),
                    true,
                );
            }
        }

        self.trace.pop();

        Ok(false)
    }

    /// Pushes the block in the specified direction, returns `Ok(true)` if the
    /// block is successfully pushed, or `Ok(false)` if the block is
    /// blocked.
    #[trace_func]
    #[instrument(skip(self, world))]
    pub fn push(&mut self, world: &World, key: BlockKey, direction: Direction) -> MoveResult<bool> {
        let block = key.get(world);
        if block.proto.is_static() {
            return Ok(false);
        }
        let position = block.state.position;
        position.container.orphan(key)?;

        let source = SourceArrow::new(position, direction, Rational::HALF);
        self.push_from(world, key, source)
    }
}

impl Algorithm {
    pub fn commit(&self, world: &mut World) {
        for movement in &self.movements {
            let block = movement.key.get_mut(world);
            block.state.position = movement.target;
        }
    }
}

impl Position {
    /// Whether the position is directly inside a void block.
    fn is_in_void(&self, world: &World) -> bool {
        self.container
            .map(|key| key.get(world).proto.is_void())
            .unwrap_or(false)
    }
}

impl BlockKey {
    /// Whether the block is void.
    fn is_void(&self, world: &World) -> bool {
        self.get(world).proto.is_void()
    }
}
