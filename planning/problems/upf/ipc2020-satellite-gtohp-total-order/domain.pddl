(define (domain strips_sat_x_1-domain)
 (:requirements :strips :typing :negative-preconditions :equality :hierarchy :method-preconditions)
 (:types satellite direction instrument mode)
 (:predicates (on_board ?i - instrument ?s - satellite) (supports ?i - instrument ?m - mode) (pointing ?s - satellite ?d - direction) (power_avail ?s - satellite) (power_on ?i - instrument) (calibrated ?i - instrument) (have_image ?d - direction ?m - mode) (calibration_target ?i - instrument ?d - direction))
 (:task do_mission
  :parameters ( ?d - direction ?m - mode))
 (:task do_prepare
  :parameters ( ?s - satellite ?i - instrument ?d - direction))
 (:task do_switching
  :parameters ( ?s - satellite ?i - instrument))
 (:task do_calibration
  :parameters ( ?s - satellite ?i - instrument ?d - direction))
 (:task make_power_available
  :parameters ( ?s - satellite ?other_i - instrument))
 (:task do_turning
  :parameters ( ?s - satellite ?d - direction))
 (:method m0_do_mission
  :parameters ( ?d - direction ?m - mode ?s - satellite ?i - instrument)
  :task (do_mission ?d ?m)
  :ordered-subtasks (and
    (t2 (do_prepare ?s ?i ?d))
    (t3 (take_image ?s ?d ?i ?m))))
 (:method m1_do_prepare
  :parameters ( ?s - satellite ?i - instrument ?d - direction)
  :task (do_prepare ?s ?i ?d)
  :ordered-subtasks (and
    (t1 (do_switching ?s ?i))
    (t2 (do_turning ?s ?d))))
 (:method m2_do_switching
  :parameters ( ?s - satellite ?i - instrument ?d - direction ?other_i - instrument)
  :task (do_switching ?s ?i)
  :precondition (and (on_board ?i ?s) (on_board ?other_i ?s) (not (power_avail ?s)))
  :ordered-subtasks (and
    (t1 (make_power_available ?s ?other_i))
    (t2 (switch_on ?i ?s))
    (t3 (do_calibration ?s ?i ?d))))
 (:method m3_do_switching
  :parameters ( ?s - satellite ?i - instrument ?d - direction)
  :task (do_switching ?s ?i)
  :precondition (and (on_board ?i ?s) (power_avail ?s))
  :ordered-subtasks (and
    (t1 (switch_on ?i ?s))
    (t2 (do_calibration ?s ?i ?d))))
 (:method m4_do_switching
  :parameters ( ?s - satellite ?i - instrument)
  :task (do_switching ?s ?i)
  :precondition (and (power_on ?i))
  :ordered-subtasks (and
    (t1 (nop ))))
 (:method m5_do_calibration
  :parameters ( ?s - satellite ?i - instrument ?d - direction)
  :task (do_calibration ?s ?i ?d)
  :precondition (and (not (calibrated ?i)))
  :ordered-subtasks (and
    (t1 (do_prepare ?s ?i ?d))
    (t2 (calibrate ?s ?i ?d))))
 (:method m6_do_calibration
  :parameters ( ?s - satellite ?i - instrument ?d - direction)
  :task (do_calibration ?s ?i ?d)
  :precondition (and (calibrated ?i))
  :ordered-subtasks (and
    (n (nop ))))
 (:method m7_make_power_available
  :parameters ( ?s - satellite ?other_i - instrument)
  :task (make_power_available ?s ?other_i)
  :precondition (and (power_on ?other_i) (not (power_avail ?s)))
  :ordered-subtasks (and
    (t1 (switch_off ?other_i ?s))))
 (:method m8_do_turning
  :parameters ( ?s - satellite ?d - direction ?other_d - direction)
  :task (do_turning ?s ?d)
  :precondition (and (pointing ?s ?other_d) (not (pointing ?s ?d)))
  :ordered-subtasks (and
    (t1 (turn_to ?s ?d ?other_d))))
 (:method m9_do_turning
  :parameters ( ?s - satellite ?d - direction)
  :task (do_turning ?s ?d)
  :precondition (and (pointing ?s ?d))
  :ordered-subtasks (and
    (n (nop ))))
 (:action turn_to
  :parameters ( ?s - satellite ?d_new - direction ?d_prev - direction)
  :precondition (and (pointing ?s ?d_prev) (not (= ?d_new ?d_prev)))
  :effect (and (pointing ?s ?d_new) (not (pointing ?s ?d_prev))))
 (:action switch_on
  :parameters ( ?i - instrument ?s - satellite)
  :precondition (and (on_board ?i ?s) (power_avail ?s))
  :effect (and (power_on ?i) (not (calibrated ?i)) (not (power_avail ?s))))
 (:action switch_off
  :parameters ( ?i - instrument ?s - satellite)
  :precondition (and (on_board ?i ?s) (power_on ?i))
  :effect (and (not (power_on ?i)) (power_avail ?s)))
 (:action calibrate
  :parameters ( ?s - satellite ?i - instrument ?d - direction)
  :precondition (and (on_board ?i ?s) (calibration_target ?i ?d) (pointing ?s ?d) (power_on ?i))
  :effect (and (calibrated ?i)))
 (:action take_image
  :parameters ( ?s - satellite ?d - direction ?i - instrument ?m - mode)
  :precondition (and (calibrated ?i) (on_board ?i ?s) (supports ?i ?m) (power_on ?i) (pointing ?s ?d))
  :effect (and (have_image ?d ?m)))
 (:action nop
  :parameters ())
)
