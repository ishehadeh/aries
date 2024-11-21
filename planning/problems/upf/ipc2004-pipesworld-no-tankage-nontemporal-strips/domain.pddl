(define (domain network1new_all_6_2_instance-domain)
 (:requirements :strips :typing)
 (:types pipe area product batch_atom)
 (:predicates (connect ?from - area ?to - area ?pipe - pipe) (unitary ?pipe - pipe) (not_unitary ?pipe - pipe) (last ?batch_atom - batch_atom ?pipe - pipe) (first ?batch_atom - batch_atom ?pipe - pipe) (follow ?next - batch_atom ?previous - batch_atom) (is_product ?batch_atom - batch_atom ?product - product) (on ?batch_atom - batch_atom ?area - area) (may_interface ?product_a - product ?product_b - product) (normal ?pipe - pipe) (push_updating ?pipe - pipe) (pop_updating ?pipe - pipe))
 (:action push_start
  :parameters ( ?pipe - pipe ?batch_atom_in - batch_atom ?from_area - area ?to_area - area ?first_batch_atom - batch_atom ?product_batch_atom_in - product ?product_first_batch - product)
  :precondition (and (normal ?pipe) (first ?first_batch_atom ?pipe) (connect ?from_area ?to_area ?pipe) (on ?batch_atom_in ?from_area) (not_unitary ?pipe) (is_product ?batch_atom_in ?product_batch_atom_in) (is_product ?first_batch_atom ?product_first_batch) (may_interface ?product_batch_atom_in ?product_first_batch))
  :effect (and (push_updating ?pipe) (not (normal ?pipe)) (first ?batch_atom_in ?pipe) (not (first ?first_batch_atom ?pipe)) (follow ?first_batch_atom ?batch_atom_in) (not (on ?batch_atom_in ?from_area))))
 (:action push_end
  :parameters ( ?pipe - pipe ?from_area - area ?to_area - area ?last_batch_atom - batch_atom ?next_last_batch_atom - batch_atom)
  :precondition (and (push_updating ?pipe) (last ?last_batch_atom ?pipe) (connect ?from_area ?to_area ?pipe) (not_unitary ?pipe) (follow ?last_batch_atom ?next_last_batch_atom))
  :effect (and (not (push_updating ?pipe)) (normal ?pipe) (not (follow ?last_batch_atom ?next_last_batch_atom)) (last ?next_last_batch_atom ?pipe) (not (last ?last_batch_atom ?pipe)) (on ?last_batch_atom ?to_area)))
 (:action pop_start
  :parameters ( ?pipe - pipe ?batch_atom_in - batch_atom ?from_area - area ?to_area - area ?last_batch_atom - batch_atom ?product_batch_atom_in - product ?product_last_batch - product)
  :precondition (and (normal ?pipe) (last ?last_batch_atom ?pipe) (connect ?from_area ?to_area ?pipe) (on ?batch_atom_in ?to_area) (not_unitary ?pipe) (is_product ?batch_atom_in ?product_batch_atom_in) (is_product ?last_batch_atom ?product_last_batch) (may_interface ?product_batch_atom_in ?product_last_batch))
  :effect (and (pop_updating ?pipe) (not (normal ?pipe)) (last ?batch_atom_in ?pipe) (not (last ?last_batch_atom ?pipe)) (follow ?batch_atom_in ?last_batch_atom) (not (on ?batch_atom_in ?to_area))))
 (:action pop_end
  :parameters ( ?pipe - pipe ?from_area - area ?to_area - area ?first_batch_atom - batch_atom ?next_first_batch_atom - batch_atom)
  :precondition (and (pop_updating ?pipe) (first ?first_batch_atom ?pipe) (connect ?from_area ?to_area ?pipe) (not_unitary ?pipe) (follow ?next_first_batch_atom ?first_batch_atom))
  :effect (and (not (pop_updating ?pipe)) (normal ?pipe) (not (follow ?next_first_batch_atom ?first_batch_atom)) (first ?next_first_batch_atom ?pipe) (not (first ?first_batch_atom ?pipe)) (on ?first_batch_atom ?from_area)))
 (:action push_unitarypipe
  :parameters ( ?pipe - pipe ?batch_atom_in - batch_atom ?from_area - area ?to_area - area ?first_batch_atom - batch_atom ?product_batch_atom_in - product ?product_first_batch - product)
  :precondition (and (first ?first_batch_atom ?pipe) (connect ?from_area ?to_area ?pipe) (on ?batch_atom_in ?from_area) (unitary ?pipe) (is_product ?batch_atom_in ?product_batch_atom_in) (is_product ?first_batch_atom ?product_first_batch) (may_interface ?product_batch_atom_in ?product_first_batch))
  :effect (and (first ?batch_atom_in ?pipe) (not (first ?first_batch_atom ?pipe)) (last ?batch_atom_in ?pipe) (not (last ?first_batch_atom ?pipe)) (not (on ?batch_atom_in ?from_area)) (on ?first_batch_atom ?to_area)))
 (:action pop_unitarypipe
  :parameters ( ?pipe - pipe ?batch_atom_in - batch_atom ?from_area - area ?to_area - area ?last_batch_atom - batch_atom ?product_batch_atom_in - product ?product_last_batch - product)
  :precondition (and (last ?last_batch_atom ?pipe) (connect ?from_area ?to_area ?pipe) (on ?batch_atom_in ?to_area) (unitary ?pipe) (is_product ?batch_atom_in ?product_batch_atom_in) (is_product ?last_batch_atom ?product_last_batch) (may_interface ?product_batch_atom_in ?product_last_batch))
  :effect (and (last ?batch_atom_in ?pipe) (not (last ?last_batch_atom ?pipe)) (first ?batch_atom_in ?pipe) (not (first ?last_batch_atom ?pipe)) (not (on ?batch_atom_in ?to_area)) (on ?last_batch_atom ?from_area)))
)
