(define (domain bw_rand_5-domain)
 (:requirements :strips :typing :negative-preconditions :hierarchy :method-preconditions)
 (:types block)
 (:predicates (on ?x - block ?y - block) (ontable ?x - block) (clear ?x - block) (handempty) (holding ?x - block))
 (:task do_put_on
  :parameters ( ?x - block ?y - block))
 (:task do_on_table
  :parameters ( ?x - block))
 (:task do_move
  :parameters ( ?x - block ?y - block))
 (:task do_clear
  :parameters ( ?x - block))
 (:method m0_do_put_on
  :parameters ( ?x - block ?y - block)
  :task (do_put_on ?x ?y)
  :precondition (and (on ?x ?y))
  :ordered-subtasks (and
    (t1 (nop ))))
 (:method m1_do_put_on
  :parameters ( ?x - block ?y - block)
  :task (do_put_on ?x ?y)
  :precondition (and (handempty))
  :ordered-subtasks (and
    (t1 (do_clear ?x))
    (t2 (do_clear ?y))
    (t3 (do_on_table ?y))
    (t4 (do_move ?x ?y))))
 (:method m2_do_on_table
  :parameters ( ?x - block ?y - block)
  :task (do_on_table ?x)
  :precondition (and (clear ?x) (handempty) (not (ontable ?x)))
  :ordered-subtasks (and
    (t1 (unstack ?x ?y))
    (t2 (put_down ?x))))
 (:method m3_do_on_table
  :parameters ( ?x - block)
  :task (do_on_table ?x)
  :precondition (and (clear ?x))
  :ordered-subtasks (and
    (t1 (nop ))))
 (:method m4_do_move
  :parameters ( ?x - block ?y - block)
  :task (do_move ?x ?y)
  :precondition (and (clear ?x) (clear ?y) (handempty) (ontable ?x))
  :ordered-subtasks (and
    (t1 (pick_up ?x))
    (t2 (stack ?x ?y))))
 (:method m5_do_move
  :parameters ( ?x - block ?y - block ?z - block)
  :task (do_move ?x ?y)
  :precondition (and (clear ?x) (clear ?y) (handempty) (not (ontable ?x)))
  :ordered-subtasks (and
    (t1 (unstack ?x ?z))
    (t2 (stack ?x ?y))))
 (:method m6_do_clear
  :parameters ( ?x - block)
  :task (do_clear ?x)
  :precondition (and (clear ?x))
  :ordered-subtasks (and
    (t1 (nop ))))
 (:method m7_do_clear
  :parameters ( ?x - block ?y - block)
  :task (do_clear ?x)
  :precondition (and (not (clear ?x)) (on ?y ?x) (handempty))
  :ordered-subtasks (and
    (t1 (do_clear ?y))
    (t2 (unstack ?y ?x))
    (t3 (put_down ?y))))
 (:action pick_up
  :parameters ( ?x - block)
  :precondition (and (clear ?x) (ontable ?x) (handempty))
  :effect (and (not (ontable ?x)) (not (clear ?x)) (not (handempty)) (holding ?x)))
 (:action put_down
  :parameters ( ?x - block)
  :precondition (and (holding ?x))
  :effect (and (not (holding ?x)) (clear ?x) (handempty) (ontable ?x)))
 (:action stack
  :parameters ( ?x - block ?y - block)
  :precondition (and (holding ?x) (clear ?y))
  :effect (and (not (holding ?x)) (not (clear ?y)) (clear ?x) (handempty) (on ?x ?y)))
 (:action unstack
  :parameters ( ?x - block ?y - block)
  :precondition (and (on ?x ?y) (clear ?x) (handempty))
  :effect (and (holding ?x) (clear ?y) (not (clear ?x)) (not (handempty)) (not (on ?x ?y))))
 (:action nop
  :parameters ())
)
